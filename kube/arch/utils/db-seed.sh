#!/usr/bin/env bash
# Execute all SQL migration and seed files in sql/*.sql into pgEdge / PostgreSQL.
set -euo pipefail

ENV="${ENV:-local}"
NS="arch"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"

# shellcheck source=./lib-runtime.sh disable=SC1091
source "$SCRIPT_DIR/lib-runtime.sh"
arch::kubeconfig_env

info() { echo "ℹ️  $*"; }
ok()   { echo "✅ $*"; }
die()  { echo "🛑 $*" >&2; exit 1; }

[ -d "$REPO_ROOT/sql" ] || die "Directory $REPO_ROOT/sql not found"

info "Waiting for database pod 'arch-database-0' in namespace '$NS' to be ready..."
kubectl -n "$NS" rollout status statefulset/arch-database --timeout=180s >/dev/null

# Ensure PostgreSQL server is ready to accept connections
info "Waiting for PostgreSQL service to accept connections..."
until kubectl -n "$NS" exec -i arch-database-0 -- pg_isready -U postgres -d arch >/dev/null 2>&1; do
  sleep 1
done

ok "Database is accepting connections."

info "Ensuring schema 'arch' exists and default search_path is set..."
kubectl -n "$NS" exec -i arch-database-0 -- psql -U postgres -d arch -v ON_ERROR_STOP=1 <<'EOSQL' >/dev/null
CREATE SCHEMA IF NOT EXISTS arch;
ALTER DATABASE arch SET search_path TO arch, public;
ALTER ROLE postgres IN DATABASE arch SET search_path TO arch, public;
ALTER ROLE postgres SET search_path TO arch, public;
EOSQL

# Files 00-50 (schema + reference data) are loaded ONCE: they are plain CREATE / INSERT. Files 51+
# (org chart, programs, admin, i18n labels) are idempotent and run every time.
psql_arch() { kubectl -n "$NS" exec -i arch-database-0 -- psql -U postgres -d arch -v ON_ERROR_STOP=1 "$@"; }
schema_loaded=""
if [ "$(psql_arch -Atc "SELECT to_regclass('arch.db_language') IS NOT NULL" 2>/dev/null)" = "t" ]; then
  schema_loaded=1
  if [ "$(psql_arch -Atc "SELECT to_regclass('arch.su_user_assignment') IS NOT NULL AND to_regclass('arch.al_data_change') IS NOT NULL AND to_regclass('arch.rp_report_job') IS NOT NULL AND to_regclass('arch.nt_notification') IS NOT NULL" 2>/dev/null)" != "t" ]; then
    die "The database was loaded from an older sql/01_ysql.sql (no su_user_assignment / al_data_change / rp_report_job / nt_notification). Recreate it: make down && make bootstrap"
  fi
  info "Schema already loaded: skipping the load-once files (00-50), re-running the idempotent seeds (51+)."
fi

# Count files
total_files=$(ls -1 "$REPO_ROOT"/sql/*.sql 2>/dev/null | wc -l)
info "Executing $total_files SQL migration and seed files in order..."

count=0
for sql_file in $(ls -1 "$REPO_ROOT"/sql/*.sql | sort -V); do
  filename="$(basename "$sql_file")"
  count=$((count + 1))
  if [ -n "$schema_loaded" ] && [ "$((10#${filename%%_*}))" -le 50 ]; then
    printf "  [%2d/%2d] %-35s ... ⏭  already loaded\n" "$count" "$total_files" "$filename"
    continue
  fi
  printf "  [%2d/%2d] Running %-35s ... " "$count" "$total_files" "$filename"
  
  if kubectl -n "$NS" exec -i arch-database-0 -- psql -U postgres -d arch -v ON_ERROR_STOP=1 < "$sql_file" >/dev/null 2>&1; then
    echo "✅ OK"
  else
    echo "❌ FAILED"
    # Re-run without silent redirect to show error details
    kubectl -n "$NS" exec -i arch-database-0 -- psql -U postgres -d arch -v ON_ERROR_STOP=1 < "$sql_file" || true
    die "Failed executing $filename"
  fi
done

echo ""
ok "All $total_files SQL files successfully applied to database 'arch' (schema: arch)!"
