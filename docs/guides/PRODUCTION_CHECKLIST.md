# Production Readiness Checklist

This repository is configured for **local development** (single-node k3d cluster, public dev credentials committed in the repo) — it is not production-ready out of the box.
This checklist enumerates everything that must be changed or decided before going to production, along with the exact files to update (check off `☐` as you complete each item).

> Current state: only the `kube/arch/overlays/local` overlay exists; `make deploy ENV=staging|production` **will not work yet** until you create `kube/arch/overlays/<env>/`; application services (`api-*`) run via `cargo` on the host and do not yet have Dockerfiles or Kubernetes manifests.

## 1. Secrets & Credentials (Must All Be Rotated and Removed from Repo)

| ☐ | Secret / Credential | Current Location | Production Action |
|---|---|---|---|
| ☐ | Postgres password (`postgres`) | `.env` (`DATABASE_URL`, `POSTGRES_PASSWORD`), `kube/arch/shared/security/*.yaml`, **Kratos DSN in ConfigMap** (`arch_identity/base/configmap.yaml`) | Move to external Secret/Vault; create least-privilege per-service DB roles (currently every service connects as superuser `postgres`) |
| ☐ | `KRATOS_SECRET_COOKIE`, `KRATOS_SECRET_CIPHER` | `.env`, `shared-credentials-secret.yaml` | Generate new random secrets (≥ 32 chars); rotating invalidates existing sessions |
| ☐ | **`login-guard` webhook secret** (`login-guard-dev-secret-…`) | `arch_identity/base/configmap.yaml` **and** `KRATOS_WEBHOOK_SECRET` on `api-master` | Must match on both sides; move to Kubernetes Secret |
| ☐ | Garage: access key/secret (`GKa4c1e…`), `GARAGE_ADMIN_TOKEN`, `GARAGE_RPC_SECRET` | `.env`, `arch_storage/base/configmap.yaml` (including the bootstrap script with embedded key/secret) | Move to Secret; issue separate keys per service; do not expose the Garage Admin API outside the cluster |
| ☐ | `REPORT_API_KEY` | `.env` | Rotate or remove if unused |
| ☐ | Dev login accounts: `admin@arch.local / Admin@2026`, `test_abac@arch.local / Test@2026` | `make seed-admin`, `make seed-test-users`, `sql/53_admin_seed.sql`, `sql/54_abac_test_seed.sql` | **Never run `seed-test-users` / `54_` in production**; provision a real admin account with a strong password and disable/remove dev accounts |
| ☐ | Verify no secrets leaked in git history | — | `.gitignore` already excludes `.env`, `.kube/`, `target/`, `dist/`; before committing or publishing, verify `git status` and rotate any credential that was ever committed |

## 2. Identity & Authentication (Ory Kratos)

| ☐ | Item | Where to Change |
|---|---|---|
| ☐ | Disable `dev: true` | `kratos.yml` in `arch_identity/base/configmap.yaml` |
| ☐ | Set `base_url`, `ui_url`, `allowed_return_urls`, and CORS to real production domains (currently `localhost`) | `kratos.yml` + `CORS_ALLOWED_ORIGINS` in `.env` |
| ☐ | **Decide on self-service registration (currently enabled)**: disable if user accounts must only be provisioned by admins via `SUPG004` | `selfservice.flows.registration` |
| ☐ | Configure cookies with `Secure` + appropriate `SameSite` over HTTPS | Kratos `serve.public` / cookies |
| ☐ | Restrict Kratos **Admin API (4434)** to cluster-internal traffic only | Currently mapped to host by k3d — see Section 3 |
| ☐ | Configure password policy / MFA / email verification & recovery over real SMTP | `kratos.yml` + `SUPG006` (in dev, recovery links can be generated manually in `SUPG004`) |
| ☐ | Ensure `api-master` is highly available with health checks (Kratos calls the `login-guard` hook on every sign-in) | `api-master` Deployment |

## 3. Networking & TLS

| ☐ | Item | Notes |
|---|---|---|
| ☐ | Terminate HTTPS on all external entry points (Traefik/Ingress + TLS certificates) | Currently plain HTTP |
| ☐ | Expose only required external ports | `k3s-up.sh` maps host ports `5432`, `6379`, `9092`, `9644`, `8080`, `4433`, **`4434`**, `3900`, **`3903`**, `7280`, `7281` — production should expose only the web/API/Kratos public ingress |
| ☐ | Enforce Kubernetes `NetworkPolicy` so DB, Valkey, Redpanda, Quickwit, and Garage only accept connections from authorized services | See `docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md` |
| ☐ | Replace the Trunk dev proxy with a production reverse proxy routing the same prefixes (`/api/kratos`, `/api/v1/notifications`, `/api/v1/reports`, `/api/v1`) | `Trunk.toml.tmpl` / `gen-trunk-config.sh` are for local development only |
| ☐ | Lock down `CORS_ALLOWED_ORIGINS` to production domains and review `CORS_ALLOW_CREDENTIALS` | `.env` |
| ☐ | Add rate limiting / WAF in front of public APIs (not built into the application binary) | Ingress / API Gateway |
| ☐ | Trust `X-Forwarded-For` only from trusted reverse proxies (audit logging records client IPs from this header) | `infrastructure/src/middleware/audit/audit_trail.rs` |

## 4. Data Persistence & Durability

| ☐ | Item | Current Dev State | Production Requirement |
|---|---|---|---|
| ☐ | **Automated backups** for Postgres, Garage, and Quickwit | None; PVCs use `local-path` bound to a single node | Scheduled backups + tested restore procedures (`pg_dump`/WAL archiving, S3 bucket snapshots) |
| ☐ | Postgres HA / multi-region (pgEdge Spock) | Single node | Follow `docs/architecture/04_MULTI_REGION_TOPOLOGY.md` |
| ☐ | Redpanda | `dev-container` mode, single node, 1 GB RAM | Production mode, replication factor ≥ 3, SASL/TLS enabled |
| ☐ | Garage S3 | `replication_factor = 1`, single node | ≥ 3 nodes/zones, lifecycle expiration policy for generated report files (currently retained indefinitely) |
| ☐ | Quickwit | Local disk metastore and index storage | Use object storage + PostgreSQL metastore; define retention policy for **`audit-logs-v1`** (`events-*` indexes retain 90 days) |
| ☐ | Schema migrations | `sql/01_ysql.sql` loaded once | Use idempotent migration scripts — see [`sql/README.md`](../../sql/README.md) |
| ☐ | Data retention & cleanup jobs | `nt_notification`, `rp_report_job` + S3 report files, and unshipped `al_data_change` rows have no automatic retention cleanup | Define retention policies + scheduled cleanup jobs |

## 5. Privacy (PDPA/GDPR) & Audit Logging

- Audit logs capture **client IP, User-Agent, request/response headers and bodies** (redacting sensitive fields such as cookies/passwords/tokens to `***`), as well as before/after snapshots of every mutated database row — review against your privacy/PDPA compliance policy and configure retention limits (☐).
- Toggle via `AUDIT_LOG_ENABLED` — disabling it leaves `ALPG000` empty and stops recording compliance audit trails.
- Restrict `ALPG000` (Audit Log) `READ` access and `ALPG001` (Process Monitor) `WRITE` (reprocess) access strictly to auditors and administrators (☐).

## 6. Application Packaging & Runtime

| ☐ | Item | Where to Change |
|---|---|---|
| ☐ | Build container images + Kubernetes manifests for `api-master`, `api-report`, and `api-notification` (resource limits, `/livez` & `/readyz` probes, graceful shutdown ≥ 10s) | New Dockerfiles / manifests |
| ☐ | Build the web frontend in release mode (`trunk build --release`) and serve as static assets; set `APP_ENV=production` so `/demo` routes are excluded (`is_dev_env()`) | `handler/web/application/master` |
| ☑ | Tailwind CSS is compiled at build time via **Tailwind CLI 3.4.17** (pinned in `Trunk.toml.tmpl`) with no runtime browser CDN, and Inter fonts are self-hosted from `handler/web/package/fonts/`. Remaining: add a strict `Content-Security-Policy` header (using nonces/hashes for the small inline theme/Trunk bootstrap scripts) | `tailwind.config.js`, `index.html` |
| ☐ | Browser-reachable S3 URL for Garage downloads: pre-signed URLs are generated from `AWS_ENDPOINT_URL` — ensure it points to the public S3 domain (or introduce a separate public endpoint variable) | `infrastructure` (S3 presign) |
| ☐ | Multi-instance scaling: each `api-notification` replica must have a unique `NOTIFICATION_CONSUMER_GROUP`, whereas `api-report` replicas share `REPORT_CONSUMER_GROUP` | `.env` / Deployment manifest |
| ☐ | Multi-region caching: Valkey instances are regional → broadcast cache invalidation events over Redpanda (currently relies on TTL across regions) | `docs/architecture/11_VALKEY_READ_CACHE.md` |
| ☐ | Production log level: set an appropriate `RUST_LOG` (dev defaults enable `debug` for `api_master` and `arch::cache`) | Environment variables |
| ☐ | Review dev toggles before launch: `READ_CACHE_ENABLED`, `CACHE_STATS_SECS` | `.env` |

## 7. Engineering Operations & CI/CD

- ☐ CI pipeline: build + `cargo test --workspace --exclude master-app` + WASM check + `kubeconform` + `cargo audit`.
- ☐ Code formatting & linting: enforce `rustfmt.toml` and `clippy.toml` in CI.
- ☐ Incident response plan: on-call rotation, backup restoration runbooks, operational troubleshooting guide (start from [`TROUBLESHOOTING.md`](./TROUBLESHOOTING.md)).
- ☐ Load & capacity testing: audit log ingestion and multi-million-row CSV exports.

## 8. Mail & Sign-In Protection

- Set `mail.smtp_url` / `mail.from` in `su_configuration` (the SMTP URL may contain credentials, so restrict read access to that table; without these settings, users will not receive password setup emails).
- Route browser requests for Kratos `/self-service/*` through `api-master` `/api/v1/auth/kratos/*` (where brute-force lockout and password policy enforcement live), and block direct internet access to Kratos' public port for login submissions.
- Apply the Kratos configuration with the `settings.after.password` webhook (`/hooks/password-changed`) and a strong `KRATOS_WEBHOOK_SECRET`.
- Outbound emails are delivered by **api-notification** (consuming the `mail-outbox` topic): ensure `api-notification` is running and the topic exists with a short retention period (`make topics`, 1 hour) — messages contain one-time password setup links and must not be indexed in Quickwit.
