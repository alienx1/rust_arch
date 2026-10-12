#!/usr/bin/env bash
# Container-runtime detection and environment setup for k3s-* scripts.
# Supports rootless/rootful Podman and Docker.
#
# Sourced, not executed — each caller sets `set -euo pipefail`.
#
# Override auto-detection with ARCH_RUNTIME=podman|docker.

arch::detect_runtime() {
  if [ -n "${ARCH_RUNTIME:-}" ]; then
    case "$ARCH_RUNTIME" in
      podman | docker) return 0 ;;
      *) echo "🛑 ARCH_RUNTIME must be 'podman' or 'docker' (got '$ARCH_RUNTIME')" >&2; return 1 ;;
    esac
  fi

  if command -v podman >/dev/null 2>&1; then
    ARCH_RUNTIME="podman"
  elif arch::_docker_is_real; then
    ARCH_RUNTIME="docker"
  else
    echo "🛑 Neither podman nor a real docker daemon found on PATH." >&2
    return 1
  fi
  export ARCH_RUNTIME
}

arch::_docker_is_real() {
  command -v docker >/dev/null 2>&1 || return 1
  ! docker --version 2>&1 | grep -qi podman
}

arch::runtime_env() {
  arch::detect_runtime || return 1
  local user_id; user_id="$(id -u)"

  if [ "$ARCH_RUNTIME" = "podman" ]; then
    ARCH_SOCK="${XDG_RUNTIME_DIR:-/run/user/$user_id}/podman/podman.sock"
    export CONTAINER_HOST="unix://$ARCH_SOCK"
    export DOCKER_HOST="unix://$ARCH_SOCK"
  else
    local configured="${DOCKER_HOST:-}"
    [[ "$configured" == *podman* ]] && configured=""

    if [ -n "$configured" ] && [[ "$configured" != unix://* ]]; then
      export DOCKER_HOST="$configured"
      ARCH_SOCK="$configured"
      echo "⚠️  DOCKER_HOST is remote ($configured); k3d needs a local socket" >&2
      return 0
    fi

    if [ -n "$configured" ]; then
      ARCH_SOCK="${configured#unix://}"
    elif [ -S "${XDG_RUNTIME_DIR:-/run/user/$user_id}/docker.sock" ]; then
      ARCH_SOCK="${XDG_RUNTIME_DIR:-/run/user/$user_id}/docker.sock"
    else
      ARCH_SOCK="/var/run/docker.sock"
    fi
    export DOCKER_HOST="unix://$ARCH_SOCK"
  fi

  export DOCKER_SOCK="$ARCH_SOCK"
}

arch::ensure_socket() {
  if [ "$ARCH_RUNTIME" = "podman" ]; then
    if [ ! -S "$ARCH_SOCK" ]; then
      echo "⏳ Starting Podman user socket service..."
      systemctl --user start podman.socket 2>/dev/null \
        || podman system service --time=0 "unix://$ARCH_SOCK" &
      sleep 2
    fi
  else
    docker info >/dev/null 2>&1 || {
      echo "🛑 Docker daemon not reachable at $DOCKER_HOST" >&2
      echo "   Start Docker (e.g. 'sudo systemctl start docker') and retry." >&2
      return 1
    }
  fi
}

arch::is_rootless() {
  if [ "$ARCH_RUNTIME" = "podman" ]; then
    [ "$(podman info --format '{{.Host.Security.Rootless}}' 2>/dev/null)" = "true" ]
  else
    docker info 2>/dev/null | grep -qi 'rootless'
  fi
}

arch::print_runtime() {
  echo "🧰 Runtime: $ARCH_RUNTIME ($( arch::is_rootless && echo rootless || echo rootful )) via $ARCH_SOCK"
}

arch::kubeconfig_env() {
  local repo_root
  repo_root="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"
  mkdir -p "$repo_root/.kube"
  export KUBECONFIG="${KUBECONFIG:-$repo_root/.kube/config}"
}
