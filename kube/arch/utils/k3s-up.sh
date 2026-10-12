#!/usr/bin/env bash
# Start the container runtime socket and create/start the local K3s cluster.
# Works on Podman (rootless or rootful) and Docker.
set -euo pipefail

CLUSTER="arch-k3s"
K3S_IMAGE="${ARCH_K3S_IMAGE:-rancher/k3s:v1.31.5-k3s1}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"

# shellcheck source=./lib-runtime.sh disable=SC1091
source "$SCRIPT_DIR/lib-runtime.sh"

arch::runtime_env
arch::kubeconfig_env
arch::ensure_socket
arch::print_runtime

if k3d cluster list 2>/dev/null | grep -q "$CLUSTER"; then
  echo "ℹ️  K3s cluster '$CLUSTER' already exists."
  k3d cluster start "$CLUSTER"
  k3d kubeconfig write "$CLUSTER" -o "$KUBECONFIG" --kubeconfig-switch-context >/dev/null
else
  echo "🚀 Creating K3s cluster '$CLUSTER' on $ARCH_RUNTIME..."

  create_args=(
    --image "$K3S_IMAGE"
    --servers 1
    --no-lb
    --api-port 6443
    --volume "$REPO_ROOT/sql:/mnt/sql@server:0"
    --port "5432:30432@server:0:direct"
    --port "6379:30379@server:0:direct"
    --port "9092:30092@server:0:direct"
    --port "9644:30644@server:0:direct"
    --port "8080:30080@server:0:direct"
    --port "4433:30433@server:0:direct"
    --port "4434:30434@server:0:direct"
    --port "3900:30900@server:0:direct"
    --port "3903:30903@server:0:direct"
    --port "7280:30280@server:0:direct"
    --port "7281:30281@server:0:direct"
    --k3s-arg "--disable=traefik@server:0"
  )

  if arch::is_rootless; then
    create_args+=(--k3s-arg "--kubelet-arg=feature-gates=KubeletInUserNamespace=true@server:0")
  fi

  k3d cluster create "$CLUSTER" "${create_args[@]}" --wait
  k3d kubeconfig write "$CLUSTER" -o "$KUBECONFIG" --kubeconfig-switch-context >/dev/null
fi

echo ""
echo "✅ Cluster status:"
kubectl cluster-info
echo ""
kubectl get nodes
