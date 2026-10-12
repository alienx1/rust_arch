#!/usr/bin/env bash
# Stop the cluster WITHOUT deleting it — containers and their Podman
# volumes survive, so all PersistentVolume data is kept.
set -euo pipefail

CLUSTER="arch-k3s"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib-runtime.sh disable=SC1091
source "$SCRIPT_DIR/lib-runtime.sh"

arch::runtime_env
arch::kubeconfig_env

echo "⏸️  Stopping K3s cluster '$CLUSTER' (data preserved)..."
k3d cluster stop "$CLUSTER"
echo "✅ Cluster stopped. Run 'make up' to start it again."
