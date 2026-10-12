{
  description = "Clean Architecture Workspace in Rust (Axum, SeaORM, Ory Kratos, Leptos)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            openssl
            zlib
            zstd
            cyrus_sasl
            sea-orm-cli
            curl
            trunk
            typst

            # rdkafka-sys / zstd-sys run bindgen, which dlopens libclang at
            # build time. Without it `cargo build` dies in the build script
            # with "Unable to find libclang".
            llvmPackages.libclang
            llvmPackages.clang

            # Kubernetes stack (kube/arch/) — the k8s alternative to
            # docker/compose. `make up`, `make bootstrap` run kube/arch/utils/*.sh
            # straight from disk, so these are only the tools they call.
            k3d
            kubectl
            kustomize
            k9s
            stern
            kubeconform
            yq-go
            jq
          ];

          # macOS specific dependencies if needed, or other common rust deps
          # pkg-config and openssl are often needed for Rust projects
          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
          ];

          shellHook = ''
            echo "Rust development environment with rustup loaded."
            echo "Run 'rustup default stable' if you haven't installed a toolchain yet."
            rustup default stable
            rustup target add wasm32-unknown-unknown

            # --- Native build dependencies --------------------------------
            # bindgen (pulled in by rdkafka-sys and zstd-sys) loads libclang
            # through this variable rather than PATH.
            export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.zstd.dev}/lib/pkgconfig:${pkgs.cyrus_sasl.dev}/lib/pkgconfig:''${PKG_CONFIG_PATH:-}"

            # `nix develop` points TMPDIR at a per-shell directory it deletes
            # on exit. A terminal that outlives its creating shell keeps the
            # stale value, and k3d then aborts cluster creation halfway through
            # trying to write its temp hosts file there.
            if [ -n "''${TMPDIR:-}" ] && [ ! -d "$TMPDIR" ]; then
              mkdir -p "$TMPDIR" 2>/dev/null || export TMPDIR=/tmp
            fi

            # --- Kubernetes stack (kube/arch/) ----------------------------
            # Point k3d at the rootless Podman socket and keep kubeconfig in
            # the repo instead of ~/.kube/. kube/arch/utils/lib-runtime.sh does
            # the same detection on its own (and also supports Docker, via
            # ARCH_RUNTIME=docker), so this only pre-warms the shell.
            if command -v podman >/dev/null 2>&1; then
              export ARCH_RUNTIME="podman"
              PODMAN_SOCK="''${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/podman/podman.sock"
              if [ ! -S "$PODMAN_SOCK" ]; then
                systemctl --user start podman.socket 2>/dev/null \
                  || podman system service --time=0 "unix://$PODMAN_SOCK" &>/dev/null &
              fi
              export DOCKER_HOST="unix://$PODMAN_SOCK"
              export CONTAINER_HOST="unix://$PODMAN_SOCK"
              # k3d's tools node needs this raw path, separate from DOCKER_HOST.
              export DOCKER_SOCK="$PODMAN_SOCK"
            fi
            mkdir -p "$PWD/.kube"
            export KUBECONFIG="''${KUBECONFIG:-$PWD/.kube/config}"
          '';
        };
      in
      {
        devShells.default = rustShell;
      }
    );
}
