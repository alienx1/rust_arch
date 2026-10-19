# Arch — Master Data & Platform (Rust 2024)

A multi-region master data and reporting platform written in Rust: **Axum** (API) + **Leptos** (WebAssembly SPA) +
**Postgres/pgEdge** + **Ory Kratos** (authentication) + **Valkey** (cache) + **Redpanda** (event streaming) + **Garage** (S3 object storage) + **Quickwit** (audit / event search).

> This document is the entry point — coding standards live in [`AGENTS.md`](./AGENTS.md), and architecture documentation lives in [`docs/`](./docs/README.md).

## Getting Started

Prerequisites: **Nix (with flakes enabled)** and **Podman (rootless) or Docker** for k3d — Rust, trunk, kubectl, k3d, kustomize, jq, etc. are provided by `flake.nix`.

```bash
cp .env.example .env        # 0. (if .env does not exist yet) Ready-to-use dev settings, including KRATOS_WEBHOOK_SECRET required for login
nix develop                 # 1. Enter the dev shell with all tools (sets up rustup + wasm32 target)
make bootstrap              # 2. k3d cluster + Postgres/Valkey/Redpanda/Kratos/Garage/Quickwit + schema/seed + login accounts
make topics                 # 3. Create Redpanda topics + event indexes in Quickwit (not run by bootstrap)
make master                 # 4. api-master :8000   (in a new terminal; must be running before signing in — see "Sign-In" below)
make web                    # 5. Web UI :3000 (trunk) → open http://localhost:3000 (first run downloads Tailwind CLI 3.4.17 automatically; requires internet)
```

Optional services when using reporting or notifications: run `make report` (:8001) and `make notification` (:8002) in separate terminals.

**Sign-In (dev credentials):** `admin@arch.local` / `Admin@2026` (full permissions) and `test_abac@arch.local` / `Test@2026` (READ-only, scoped to the IT department).
Kratos invokes the `login-guard` hook on `api-master` (:8000) before allowing sign-in → **if `api-master` is not running, sign-in will return HTTP 500**.

> All values in `.env` and `kube/` are **public development defaults** — never use them in production. See [`docs/guides/PRODUCTION_CHECKLIST.md`](./docs/guides/PRODUCTION_CHECKLIST.md).

## Repository Map

```text
domain/                  Sea-ORM entities (domain/src/entity) + value objects — zero knowledge of DB drivers or HTTP
application/
  core/                  Ports (traits): cache, event, content(S3), audit, notification, read_cache, etc.
  master/                api-master use cases: features/<system>/<program>/{commands,queries,models,errors}
  report/                Report worker + use cases (RPPG000)
  notification/          Ingest, store, and push notifications (NTPG000)
infrastructure/          Concrete adapters: Postgres, Valkey, Redpanda, Garage S3, Quickwit, Kratos + middleware
handler/
  api/{master,report,notification}/   Axum: routers/ → call Commands traits only
  web/package/           Shared UI package (DataTable, DateRangePicker, layout, i18n, auth)
  web/application/master/  Leptos SPA: pages/<system>/<program>/{models,services,providers,components,screens}
sql/                     Schema + seeds (executed in numeric filename order) — see sql/README.md
kube/arch/               k3d/k8s manifests (kustomize `local` overlay) + utility scripts in utils/
docs/                    architecture/ (12 chapters), programs/ (per-program specs), guides/ (hands-on guides)
```

**Dependencies point strictly inward:** `handler → application → domain`, while `infrastructure` implements ports from `application-core` (full rules in `AGENTS.md`).

## Service Communication Overview

```text
Web (Leptos) ──HTTP──► api-master ──► Valkey (read cache) ──► Postgres
   │                       │  └─ audit ──► Quickwit (audit-logs-v1)         ◄── ALPG000 view/search
   │                       └─ report.requests ──► Redpanda ──► api-report ──► Garage (CSV)
   │                                                              └─ notifications ──► api-notification ──SSE──► Web (bell)
   └─ Trunk dev proxy: /api/kratos→4433, /api/v1/notifications→8002, /api/v1/reports→8001, /api/v1→8000
      (generated from `.env` during `make web`: WEB_PORT, KRATOS_PUBLIC_URL, API_*_URL — see `Trunk.toml.tmpl`)
```

Full port matrix: [`docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md`](./docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md)

## Common Commands

| Goal | Command |
|---|---|
| List all make targets | `make help` |
| Check cluster status / pod logs | `make status` / `make logs APP=arch-database` |
| Reload schema + seeds (and flush cache) | `make seed` |
| Flush Valkey cache only | `make cache-flush` |
| Check compilation / run tests | `cargo check --workspace` / `cargo test --workspace --exclude master-app` |
| Check web frontend (WASM) | `cargo check -p master-app --target wasm32-unknown-unknown` |
| Open `psql` shell (without local `psql`) | `kubectl -n arch exec -it arch-database-0 -- psql -U postgres -d arch` |
| Stop / start cluster while keeping data | `make stop` / `make up` |
| **Delete cluster and all data** | `make down` |

> `master-app` (web SPA) is excluded from workspace `cargo test` — verify it via the WASM check and in a browser.
> Setting `CARGO_TARGET_DIR` outside the repo (e.g. `~/.cache/arch_cargo_target`) is recommended — `target/` can grow to tens of GBs.

## Next Steps & Guides

| Goal | Document |
|---|---|
| Add a new screen/program (DB → API → Web → Permissions → i18n → Docs) | [`docs/guides/ADD_A_PROGRAM.md`](./docs/guides/ADD_A_PROGRAM.md) |
| Add a new service (Redpanda worker / standalone API) | [`docs/guides/ADD_A_SERVICE.md`](./docs/guides/ADD_A_SERVICE.md) |
| Troubleshoot common issues | [`docs/guides/TROUBLESHOOTING.md`](./docs/guides/TROUBLESHOOTING.md) |
| Modify database schema | [`sql/README.md`](./sql/README.md) |
| Prepare for production deployment | [`docs/guides/PRODUCTION_CHECKLIST.md`](./docs/guides/PRODUCTION_CHECKLIST.md) |
| Add caching / inspect cached namespaces | [`docs/architecture/11_VALKEY_READ_CACHE.md`](./docs/architecture/11_VALKEY_READ_CACHE.md) |
| Browse all programs and screen specs | [`docs/programs/00_INDEX.md`](./docs/programs/00_INDEX.md) |

Using an AI coding assistant? Have it read [`AGENTS.md`](./AGENTS.md) first (invariant rules: dependency layers, ban on generic filenames, tests strictly inside `tests/`, UUIDv7, etc.).

## License

[MIT](./LICENSE) © 2026 attaporn_j <alienx1step@gmail.com> — personal project; free to use, modify, and redistribute under the terms of the MIT License.
External services run alongside the platform (Garage, Quickwit, Redpanda, Ory Kratos, Valkey, Postgres) are third-party software governed by their respective licenses and are not covered by this license.
