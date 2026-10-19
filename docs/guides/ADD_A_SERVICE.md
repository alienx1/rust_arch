# Guide: Add a New Service (Redpanda Worker and/or Standalone API)

Existing services in the repository that serve as reference implementations:

| Service | Responsibility | Crates |
|---|---|---|
| **api-report** (:8001) | Worker consuming `report.requests` → generates CSV files → uploads to Garage S3; REST API to list/preview/download reports | `application/report`, `handler/api/report` |
| **api-notification** (:8002) | Consumes `notifications` → stores in DB → pushes real-time updates to browsers over SSE | `application/notification`, `handler/api/notification` |
| **api-master** (:8000) | Primary API for master data, permissions, and audit logs | `application/master`, `handler/api/master` |

## 0. Should It Be a Separate Service or Part of `api-master`?

| Create a Separate Service When | Keep in `api-master` When |
|---|---|
| Running long background jobs (file generation, email delivery, bulk processing) | Performing standard synchronous CRUD / data queries |
| Needs independent scaling or restarts decoupled from the core API | Shares the exact same tables and permission checks as existing master programs |
| Holds long-lived connections (SSE, WebSockets) | Serves short-lived HTTP requests |
| A failure or crash must not impact `api-master` availability | |

Golden Rule: **Services communicate asynchronously through Redpanda (or shared database tables) — never via synchronous service-to-service HTTP calls**, and every published event message carries its own unique ID.

## 1. Shared Contract — `application/core/src/traits/<name>.rs`

```rust
pub const MY_TOPIC: &str = "my.requests";          // Redpanda topic name
pub const MY_WORKER_GROUP: &str = "my-worker";     // Consumer group ID

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MyMessage { pub job_id: String, /* self-contained payload required by the worker */ }
```

- Messages must be **self-contained** (the worker should not need to call back to the producer) and carry a UUIDv7 identifier used as the Kafka message key.
- Any fields added later must use `#[serde(default)]` so older messages in the topic remain deserializable (see `reprocess` on `ReportJobMessage`).
- Re-export the module in `traits/mod.rs`.
- Producer side (e.g. `api-master`): call `EventService::publish(topic, Some(&id), &bytes)` — if the broker is unreachable, return HTTP 503 rather than pretending the job succeeded.

## 2. Application Use Cases — `application/<svc>/`

```text
application/<svc>/
├── Cargo.toml                  # name = "application-<svc>"  (add to root [workspace] members + [workspace.dependencies])
└── src/features/<program>/     # Same CQRS layout as ADD_A_PROGRAM (commands/queries/models/errors)
    └── commands/<x>_handler.rs # struct …Handler: impl EventHandler → parses JSON → invokes executor
```

- `EventHandler::handle(key, payload)`: **must never panic and does not return an error** — the consumer always commits the offset after `handle` returns (at-least-once delivery, preventing poison-message loops). Consequently, the handler must **record its own failure state** (e.g. updating the job row to `failed` with an error message) and be **idempotent** (duplicate messages must be safely ignored — see `run_report_job.rs`, which skips already-completed jobs).
- Unparseable messages: log a warning and skip (retrying malformed JSON will never succeed).
- Write unit tests using in-memory mocks for all ports (`application/report/tests/rppg000_test.rs` uses in-memory storage, audit, and notification mocks).
- To notify a user upon completion: call `NotificationService::notify(username, Notification{…})` (publishes to the `notifications` topic for `api-notification` to persist and push via SSE; fire-and-forget so it never blocks the job).

## 3. Binary Crate — `handler/api/<svc>/`

Model after `handler/api/report/` (include `[lib]` + `src/lib.rs` so integration tests in `tests/` can import modules):

| File | Responsibility |
|---|---|
| `Cargo.toml` | `name = "api-<svc>"`, `[lib] name = "api_<svc>"`, `[[bin]]`; register in root `Cargo.toml` |
| `src/main.rs` | Load `.env`, initialize tracing, read `<SVC>_HOST/PORT/PREFIX`, configure **graceful shutdown + 10-second watchdog + immediate exit on second Ctrl+C** (prevents open SSE streams from hanging shutdown) |
| `src/connection.rs` | Connect to DB / Valkey / Kratos / required adapters; assemble command executors; launch `spawn_worker` |
| `src/routers/base_router.rs` | `/health`, `/livez`, `/readyz`; wrap program routes with `require_kratos_session`; `cbor_error_responses`; CORS; tracing |
| `src/routers/<program>_router.rs` | Axum router consuming `State<Arc<dyn …Commands>>` only |

Background worker loop (modeled on `spawn_report_worker`):

```rust
tokio::spawn(async move {
    let consumer = RedpandaEventConsumer::new(&brokers);          // commits offset after handler completes
    let handler: Arc<dyn EventHandler> = Arc::new(MyHandler::new(commands));
    loop {
        if let Err(e) = consumer.consume(MY_TOPIC, MY_WORKER_GROUP, handler.clone()).await {
            error!(error = %e, "worker stopped, retrying in 5 s");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;         // reconnect on broker disconnect
    }
});
```

**Consumer group strategy:** Multiple instances that **share a work queue** use the same consumer group ID (each message is processed by one instance). Instances that **must each receive every message** (e.g. pushing SSE events to connected browser sessions) must use a unique consumer group per instance (see `NOTIFICATION_CONSUMER_GROUP`).

## 4. Platform Integration

| Task | Where |
|---|---|
| Dev Make target | `Makefile`: add `run-<svc>` / alias / (`dev-<svc>`) and list in `.PHONY` |
| Environment variables | `.env` and `.env.example`: `<SVC>_HOST`, `<SVC>_PORT`, `<SVC>_PREFIX`, (`<SVC>_CONSUMER_GROUP`) |
| Redpanda topic | `Makefile` target `topics`: add topic name to the loop (idempotent) — note that `make bootstrap` does **not** create topics |
| Web dev proxy | `handler/web/application/master/Trunk.toml.tmpl` (**not** `Trunk.toml`, which is generated from the template + `.env` during `make web`): add a `[[proxy]]` block for `/api/v1/<svc>/` → `@@<SVC>_URL@@@@<SVC>_PREFIX@@/` **before** the catch-all `/api/v1/` entry (longer prefixes must come first), and add the variables to `gen-trunk-config.sh` and `.env.example` |
| Ports & security matrix | `docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md` |
| Inspect events in `ALPG000` | Add a Quickwit index for the topic (config under `kube/arch/applications/arch_logging/indexes/` + index name in `make event-indexes`) and add the kind to `audit_kind` (see `event_stream_query.rs`) if you want topic events searchable in the Audit Log UI |
| Process monitoring / retry | If job status is tracked in a database table, consider exposing it in `ALPG001` (Process Monitor) |

> In development, all application services run **on the host** via `cargo`, while Postgres, Valkey, Redpanda, Kratos, Garage, and Quickwit run inside k3d.
> This repository **does not yet include Dockerfiles or Kubernetes manifests for the Rust application services** — add them when deploying into a cluster (see [`PRODUCTION_CHECKLIST.md`](./PRODUCTION_CHECKLIST.md)).

## 5. Adding New Infrastructure Adapters (When Introducing New Technology)

1. Define the **port (trait)** in `application/core/src/traits/` using domain language, never driver-specific types.
2. Implement the adapter in `infrastructure/src/services/<name>/` (one file per operation: `get.rs`, `del_prefix.rs`, …).
3. Shared middleware across services lives in `infrastructure/src/middleware/` organized by concern: `auth/` (API key, Kratos), `audit/`, `cache/`, `http/` (CORS, language negotiation, error translation) — place new middleware in the matching folder and re-export in its `mod.rs` and `middleware/mod.rs`.
4. `application/*` crates must **never** import infrastructure drivers directly.

## 6. Verification Checklist

- [ ] Publishing a message to the topic triggers the worker: `kubectl -n arch exec deploy/arch-streaming -- rpk topic consume <topic> -n 5`
- [ ] Duplicate or malformed messages do not hang the worker or cause duplicate side effects (covered by unit tests)
- [ ] Pressing `Ctrl+C` shuts down cleanly within a few seconds (even with active SSE connections)
- [ ] Restarting Redpanda causes the worker to reconnect automatically (logs `"retrying in 5 s"`)
- [ ] Web UI requests route properly through the Trunk proxy (`curl localhost:3000/api/v1/<svc>/…` with a session cookie)
- [ ] Users cannot access each other's resources (ownership checks return `404` rather than `403` to avoid leaking resource existence — same pattern as `api-report`)
- [ ] Updated `docs/architecture/09_…` and `docs/programs/…`

## 7. Common Pitfalls

| Symptom | Cause / Fix |
|---|---|
| Service hangs on `Ctrl+C` | Open SSE streams prevent `with_graceful_shutdown` from completing → close all active streams (`close_all`) and keep the shutdown watchdog timer |
| Garage pre-signed URL fails with `"Invalid signature"` through dev proxy | The dev proxy follows HTTP redirects and rewrites the `Host` header → return **JSON `{url}`** from the API and let the browser open the pre-signed URL directly instead of redirecting |
| Topic disappears after Redpanda pod recreation | Redpanda now uses a PVC; if recreating the entire cluster from scratch, run `make topics` |
| Old messages are reprocessed when changing consumer group name | `RedpandaEventConsumer` sets `auto.offset.reset=earliest` — a brand-new consumer group reads from the beginning of the topic; keep handlers idempotent and preserve consumer group names across deployments |
| Sub-router `nest("/")` fails to match through proxy | Mount under an explicit sub-path such as `/items` |
