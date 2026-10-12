#!/usr/bin/env bash
# One-shot bring-up of the whole local stack:
#   1. K3s cluster (k3d on rootless Podman / Docker)
#   2. Application manifests (Database, Valkey, Redpanda, Kratos, Garage, Quickwit)
#   3. Database schema + seed data (sql/*.sql, in file-name order)
#   4. Login identities in Ory Kratos (admin@arch.local, test_abac@arch.local)
#
# Afterwards `make run-master` + `make run-web` is all that is left: sign in as
# admin@arch.local / Admin@2026.
set -euo pipefail

ENV="${ENV:-local}"
NS="arch"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"
OVERLAY="$REPO_ROOT/kube/arch/overlays/$ENV"

cd "$REPO_ROOT"

step() { echo ""; echo "════ $* ════"; }
info() { echo "ℹ️  $*"; }
ok()   { echo "✅ $*"; }
die()  { echo "🛑 $*" >&2; exit 1; }

[ -d "$OVERLAY" ] || die "No overlay for ENV=$ENV (expected $OVERLAY)"

mkdir -p "$REPO_ROOT/.kube"
export KUBECONFIG="${KUBECONFIG:-$REPO_ROOT/.kube/config}"

if [ "$ENV" = "local" ]; then
  step "1/4  K3s cluster"
  "$SCRIPT_DIR/k3s-up.sh"
  kubectl wait --for=condition=Ready node --all --timeout=180s >/dev/null
  ok "Node ready"
else
  step "1/4  Cluster (external)"
  ctx="$(kubectl config current-context 2>/dev/null || true)"
  [ -n "$ctx" ] || die "No kubectl context set."
  info "ENV=$ENV deploying to context: $ctx"
fi

step "2/4  Deploying manifests for ENV=$ENV"
kustomize build "$OVERLAY" --load-restrictor LoadRestrictionsNone | kubectl apply -f -
ok "Manifests applied successfully"

step "3/4  Applying database schema & seeds (sql/*.sql)"
sleep 3
"$SCRIPT_DIR/db-seed.sh"

step "4/4  Login identities (Ory Kratos)"
if [ "$ENV" != "local" ]; then
  info "ENV=$ENV: create the Kratos identities yourself (make seed-admin targets localhost:4434)"
else
# su_user rows (sql/53_, 54_) are the application side of an account; the credential lives in
# Kratos. Kratos' admin API is published on localhost:4434 by the k3d port mapping.
info "Waiting for Kratos to accept admin requests..."
kratos_ready=""
for _ in $(seq 1 90); do
  if curl -fsS -o /dev/null "http://localhost:4434/admin/health/ready" 2>/dev/null; then
    kratos_ready=1
    break
  fi
  sleep 2
done
if [ -n "$kratos_ready" ]; then
  make --no-print-directory seed-admin seed-test-users
else
  echo "⚠️  Kratos did not become ready in time; once it is up run: make seed-admin seed-test-users"
fi
fi

echo ""
echo "🎉 Bootstrap complete! Run 'make status' to inspect running pods."
