#!/usr/bin/env bash
# Show k3d cluster and Kubernetes node status.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib-runtime.sh disable=SC1091
source "$SCRIPT_DIR/lib-runtime.sh"

arch::runtime_env
arch::kubeconfig_env

echo "📦 k3d Clusters:"
k3d cluster list
echo ""
echo "☸️  Kubernetes Nodes:"
kubectl get nodes -o wide 2>/dev/null || echo "Cluster unreachable or not running."
echo ""
echo "🚀 Pods in 'arch' namespace:"
kubectl get pods -n arch -o wide 2>/dev/null || true
echo ""
echo "🔌 Services in 'arch' namespace:"
kubectl get svc -n arch 2>/dev/null || true
