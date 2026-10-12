#!/usr/bin/env bash
# Writes Trunk.toml (the dev server config with its reverse proxies) from Trunk.toml.tmpl and the
# root .env, so where the web app proxies /api/* to is configured in one place.
#
# Reads (all optional; the defaults match .env.example):
#   WEB_PORT                  port of the dev server                      (3000)
#   KRATOS_PUBLIC_URL         Ory Kratos public API                       (http://localhost:4433)
#   API_MASTER_URL            api-master                                  (http://localhost:$MASTER_PORT | 8000)
#   API_REPORT_URL            api-report                                  (http://localhost:$REPORT_PORT | 8001)
#   API_NOTIFICATION_URL      api-notification                            (http://localhost:$NOTIFICATION_PORT | 8002)
#   REPORT_PREFIX / NOTIFICATION_PREFIX   path the service serves under   (/api/v1/reports, /api/v1/notifications)
#
# Usage: ./gen-trunk-config.sh [path-to-.env]      (make web calls it for you)
set -euo pipefail

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
env_file="${1:-$here/../../../../.env}"
template="$here/Trunk.toml.tmpl"
out="$here/Trunk.toml"

[ -f "$template" ] || { echo "missing $template" >&2; exit 1; }
if [ -f "$env_file" ]; then
  set -a; . "$env_file"; set +a
fi

# no trailing slash on URLs, exactly one leading slash on prefixes
url()    { printf '%s' "${1%/}"; }
prefix() { local p="${1#/}"; printf '/%s' "${p%/}"; }

WEB_PORT="${WEB_PORT:-3000}"
KRATOS_URL="$(url "${KRATOS_PUBLIC_URL:-http://localhost:4433}")"
MASTER_URL="$(url "${API_MASTER_URL:-http://localhost:${MASTER_PORT:-${API_MASTER_PORT:-8000}}}")"
REPORT_URL="$(url "${API_REPORT_URL:-http://localhost:${REPORT_PORT:-${API_REPORT_PORT:-8001}}}")"
NOTIFICATION_URL="$(url "${API_NOTIFICATION_URL:-http://localhost:${NOTIFICATION_PORT:-8002}}")"
REPORT_PREFIX="$(prefix "${REPORT_PREFIX:-${API_REPORT_PREFIX:-/api/v1/reports}}")"
NOTIFICATION_PREFIX="$(prefix "${NOTIFICATION_PREFIX:-/api/v1/notifications}")"

case "$WEB_PORT" in ''|*[!0-9]*) echo "WEB_PORT must be a number, got '$WEB_PORT'" >&2; exit 1;; esac

sed \
  -e "s|@@WEB_PORT@@|$WEB_PORT|g" \
  -e "s|@@KRATOS_URL@@|$KRATOS_URL|g" \
  -e "s|@@MASTER_URL@@|$MASTER_URL|g" \
  -e "s|@@REPORT_URL@@|$REPORT_URL|g" \
  -e "s|@@NOTIFICATION_URL@@|$NOTIFICATION_URL|g" \
  -e "s|@@REPORT_PREFIX@@|$REPORT_PREFIX|g" \
  -e "s|@@NOTIFICATION_PREFIX@@|$NOTIFICATION_PREFIX|g" \
  "$template" > "$out.tmp"

if grep -qE '@@[A-Z_]+@@' "$out.tmp"; then
  echo "unreplaced placeholder in $template:" >&2; grep -nE '@@[A-Z_]+@@' "$out.tmp" >&2; rm -f "$out.tmp"; exit 1
fi
mv "$out.tmp" "$out"
echo "Trunk.toml: web :$WEB_PORT | kratos $KRATOS_URL | master $MASTER_URL | report $REPORT_URL$REPORT_PREFIX | notification $NOTIFICATION_URL$NOTIFICATION_PREFIX"
