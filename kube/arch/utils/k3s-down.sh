#!/usr/bin/env bash
# Delete the local K3s cluster.
set -euo pipefail

CLUSTER="arch-k3s"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib-runtime.sh disable=SC1091
source "$SCRIPT_DIR/lib-runtime.sh"

arch::runtime_env
arch::kubeconfig_env

echo "🛑 Deleting K3s cluster '$CLUSTER' (this destroys all PV data)..."
k3d cluster delete "$CLUSTER" || true
echo "✅ Cluster deleted."
