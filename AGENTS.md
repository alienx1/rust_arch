# 🤖 AGENTS.md — AI Agent Engineering Guide & System Standards

> **Authoritative operational manual for AI coding assistants** (Google Antigravity, Claude, Copilot, Cursor, etc.) working inside this repository.
> Every agent working on this codebase **MUST** read, respect, and strictly enforce the patterns defined herein.

---

## 🎯 1. High-Level System Mission & Tech Stack

This platform is a high-performance, active-active multi-region master data management and reporting system built in **Rust 2024 Edition**.

- **Language / Edition**: Rust `2024` (MSRV: `1.98.0+`)
- **HTTP Engine**: `axum` (v0.8) with `tower` and `tower-http`
- **Database & ORM**: PostgreSQL 17 / pgEdge Spock wire-protocol via `sea-orm` (v2.0)
- **Frontend SPA**: Leptos (v0.8, CSR WebAssembly) + Tailwind CSS + shadcn/ui
- **Identity & Auth**: Ory Kratos (`ory-kratos-client`) with session cookies
- **Cache & Storage**: Valkey / Redis RESP (`redis`), Garage S3 (`aws-sdk-s3`)
- **Event Streaming & Logs**: Redpanda / Kafka (`rdkafka`), Quickwit / OTLP Tracing

---

## ⚡ 2. The 8 Invariant Golden Rules for Agents

### 1. Strict Layer Dependencies (Clean Architecture)
Dependencies flow **only inward**:
- `domain` ➔ Has zero knowledge of databases or HTTP. Houses Sea-ORM entities (`domain/src/entity`) and pure domain value objects.
- `application/*` ➔ Houses business logic, CQRS commands, queries, and trait contracts. **NEVER** imports `handler/*` or concrete driver implementations directly.
- `infrastructure` ➔ Houses technical adapters implementing `application-core` ports (PostgreSQL connection, Valkey cache, Redpanda publisher, Kratos middleware).
- `handler/*` ➔ Delivery tier (`api/*` for Axum REST APIs, `web/*` for Leptos WASM apps).

### 2. Feature Structure (CQRS Pattern)
Every business feature in `application/<service>/src/features/<feature>/` must be structured as follows:
```text
features/<feature>/
├── mod.rs                        # Feature scope definition & trait re-exports
├── errors/mod.rs                 # Feature-specific error enum (thiserror)
├── models/mod.rs                 # Re-exports domain entities & feature DTOs
├── commands/                     # Writes & Actions (and read-facades for routes)
│   ├── mod.rs                    # Trait: FeatureCommands & CommandHandler
│   ├── <action>_<target>.rs      # Command DTO & Handler struct
│   └── <feature>_executor.rs     # Executor implementing FeatureCommands
└── queries/                      # Reads & Lookups
    ├── mod.rs                    # Trait: FeatureQueries & QueryHandler
    ├── <action>_<target>.rs      # Query DTO & Handler struct
    └── <target>_queries.rs       # Direct Sea-ORM query runner
```

### 3. STRICT BAN on Generic File Names
- ❌ **NEVER** use `service.rs`, `manager.rs`, `handler.rs`, or `utils.rs`.
- ✅ **ALWAYS** use domain- and action-specific names that immediately convey intent:
  - `db_label_queries.rs` (Sea-ORM queries executing on `db_label`)
  - `i18n_executor.rs` (Coordinates and executes i18n operations)
  - `dashboard_service.rs` (Frontend HTTP fetcher for dashboard summary)
  - `dashboard_context.rs` (Frontend reactive Leptos context provider)

### 4. No Unnecessary Repository Layers
- When queries can be executed cleanly using Sea-ORM entities (`db_<table_name>::Entity`), do **NOT** invent a redundant repository layer.
- Execute Sea-ORM queries directly inside `queries/<target>_queries.rs`.

### 5. Routers MUST Depend on Command Trait Interfaces
- Axum route handlers in `handler/api/<service>/src/routers/` must consume:
  ```rust
  pub async fn handler(
      State(commands): State<Arc<dyn FeatureCommands>>,
      ...
  )
  ```
- Wire the executor in `handler/api/<service>/src/wiring/<feature>.rs` (`register(cx, services)`; one line in `wiring/mod.rs`): the registry resolves `State<Arc<dyn FeatureCommands>>` by type through one blanket `FromRef<AppConnection> for Arc<T>`. Do not add a field or a per-feature `FromRef` to `connection.rs` (shared file: merge conflicts).
- Route handlers must **never** receive raw `DatabaseConnection` or call query implementations directly.

### 6. Tests MUST NEVER Be Inline Inside `src`
- ❌ **NEVER** write `#[cfg(test)] mod tests { ... }` inside files in `src/`.
- ✅ **ALWAYS** place test files in an isolated `tests/` directory at the same level as `src/`:
  - `handler/api/master/tests/<feature>_router_test.rs`
  - `application/master/tests/<feature>_test.rs`
- Binary crates must configure `[lib]` in `Cargo.toml` and provide `src/lib.rs` to allow `tests/` to import modules cleanly.

### 7. Identifier Standards, Wire Format & Axum Syntax
- **UUIDs**: Always use **UUIDv7** (`Uuid::now_v7()`), never UUIDv4.
- **Axum 0.8 Path Syntax**: Always use `{param}` (e.g. `/{lid}` or `/i18n/{lid}`), never `/:lid`.
- **CBOR Wire Format**: Negotiated by `Accept` (response) and `Content-Type` (request), never Base64:
  - `application/cbor` → **raw binary CBOR** (default for Web/WASM clients: `RequestBuilder::cbor`).
  - `text/cbor-hex` → **lowercase hex string** of the same CBOR (e.g. `81a8626964...`) for curl/debugging (`RequestBuilder::cbor_hex`).
  - Anything else → JSON. Routers use `WireFormat::respond` / `WireBody<T>`; error bodies follow the same negotiation via `cbor_error_responses`.

### 8. Frontend Feature Structure (Clean UI Pattern)
Every page feature in `handler/web/application/<app>/src/pages/<feature>/` **MUST** follow the 5-layer layout:
```text
pages/<feature>/
├── mod.rs                        # Re-exports: pub use screens::<feature>_screen::<Feature>Screen as <Feature>Page;
├── models/                       # Page DTOs, ViewModels & response types (<feature>_dto.rs)
├── services/                     # HTTP API callers, env detectors, or mock loaders (<feature>_service.rs)
├── providers/                    # Reactive Leptos Context providers & hooks (<feature>_context.rs)
├── components/                   # Presentational UI widgets isolated to this feature (<widget_name>.rs)
└── screens/                      # Screen orchestrator & layout assembly (<feature>_screen.rs)
```
- **STRICT BAN on generic files**: Use `<feature>_dto.rs`, `<feature>_service.rs`, `<feature>_context.rs`, `<widget_name>.rs`, `<feature>_screen.rs`.
- **Tables**: list tables MUST use the shared `web_package::ui::component::DataTable` (header with search/refresh/actions, `FilterChips`, loading/empty states, numbered pagination) — same look as `/su/supg005`. Pages supply only `DataColumn`s and the `<TableRow>`s; do not hand-roll a `Card` + `Table` shell.
- **Optimistic locking**: edit commands carry the `row_version` the form loaded and save through `features::optimistic_update::update_versioned` (never compare or `+ 1` the version by hand); a stale save is `VersionConflict` → HTTP 412 `VERSION_CONFLICT`. See `docs/architecture/05_DATA_CONSISTENCY_AND_STORAGE.md` §2.2.
- **Two-level decision for data-changing buttons**: every button that changes data (save, delete, suspend/reactivate, revoke, force change, send mail, re-process, mark all read ...) MUST ask for confirmation before it acts: use `web_package::ui::component::{ConfirmButton, SaveConfirmButton, DeleteConfirmButton}` (first click shows "are you sure?" in place, second click runs the action). Never call a mutating callback straight from `on_click`. Buttons that only navigate, filter, page, preview or download are exempt.
- **Change notices are automatic**: a create / update / delete on a program API notifies the user who made it (kind `data_created|updated|deleted|failed`, through the notification SSE stream, toast on every page; a success is live only (never stored or indexed), a failure is kept in the bell) via the `change_notice` middleware. Do not add per-page notification code; silence a program that reports on its own in `QUIET_PROGRAMS`.
- **Environment Gating for Static Demos**: Interactive demo / playground pages (`/demo`) must be conditionally rendered only in `localhost` or `dev` environments via `is_dev_env()`, both in route definitions and in `AppShell` sidebars.

---

## 🛠️ 3. How to Implement a New Feature (Step-by-Step)

> **Complete & up-to-date guides**: [`docs/guides/ADD_A_PROGRAM.md`](docs/guides/ADD_A_PROGRAM.md) (end-to-end screen + API: DB, menu/permissions/i18n seeds, cache, web, tests, docs) and [`docs/guides/ADD_A_SERVICE.md`](docs/guides/ADD_A_SERVICE.md) (new worker/service). The steps below are a brief summary — if they conflict, follow the guides.

### A. Backend Application & Router Implementation
When asked to build a new backend feature (e.g. `department` or `employee`):

1. **Entities**:
   - Check `domain/src/entity/`. If the entity does not exist, run `make entity` or `sea-orm-cli gen entity` as configured in the [`Makefile`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/Makefile).
2. **Feature Module in `application/master/src/features/<feature>/`**:
   - Define error enum in `errors/mod.rs` with `#[derive(thiserror::Error)]`.
   - Re-export entities and define DTOs in `models/mod.rs`.
   - Define `FeatureQueries` trait in `queries/mod.rs` and implement in `queries/<target>_queries.rs` using Sea-ORM.
   - Define `FeatureCommands` trait in `commands/mod.rs` and implement in `commands/<feature>_executor.rs`.
3. **Tests for Application Layer**:
   - Create tests in `application/master/tests/<feature>_test.rs` (using mock queries/commands).
4. **Router in `handler/api/master/src/routers/`**:
   - Add `FeatureApiError` with `impl IntoResponse`.
   - Write handler functions consuming `State(commands): State<Arc<dyn FeatureCommands>>`.
   - Expose `pub fn <feature>_routes() -> Router<AppConnection>`.
   - Mount in `routers/base_router.rs`.
   - Wire it: `wiring/<feature>.rs` (build the executor from `WiringContext`, `services.insert::<dyn FeatureCommands>(..)`) + one line in `wiring/mod.rs::register_all` (after any feature it builds on); add the type to `tests/wiring_test.rs`.
5. **Tests for Router**:
   - Create tests in `handler/api/master/tests/<feature>_router_test.rs`.
6. **Verify Compilation & Tests**:
   - Run `CARGO_TARGET_DIR=/tmp/cargo_target cargo test --workspace`.
   - Ensure 0 errors, 0 warnings.

### B. Frontend Page Implementation (`handler/web/application/<app>/src/pages/<feature>/`)
When asked to build or refactor a frontend UI page:

1. **Models (`models/`)**: Define strongly-typed DTOs and state structs in `models/<feature>_dto.rs` (deriving `Clone, Debug, PartialEq, Serialize, Deserialize`).
2. **Services (`services/`)**: Implement HTTP API caller (`gloo_net::http::Request`) in `services/<feature>_service.rs`.
3. **Providers (`providers/`)**: Create reactive context (`provide_<feature>_context()`) and accessor hook (`use_<feature>()`) in `providers/<feature>_context.rs`.
4. **Components (`components/`)**: Build presentational UI components in `components/<widget_name>.rs` and re-export in `components/mod.rs`.
5. **Screens (`screens/`)**: Assemble layout (`AppShell`, breadcrumbs, responsive grids, error boundaries) in `screens/<feature>_screen.rs`.
6. **Re-export & Mount**: Re-export in `mod.rs` as `<Feature>Page`, register route in `src/app.rs`, and integrate into `AppShell` navigation (gate with `is_dev_env()` if dev-only).

---

## 🏃 4. Agent Execution & Build Commands

When executing terminal commands inside sandbox environments:
```bash
# 1. Check workspace compilation without write locks on target/
CARGO_TARGET_DIR=/tmp/cargo_target cargo check --workspace

# 2. Run all workspace test suites
CARGO_TARGET_DIR=/tmp/cargo_target cargo test --workspace

# 3. Test a single package
CARGO_TARGET_DIR=/tmp/cargo_target cargo test -p api-master
CARGO_TARGET_DIR=/tmp/cargo_target cargo test -p application-master

# 4. Generate Sea-ORM entities from PostgreSQL
make entity
```

---

## 📖 5. Key Architectural Documentation Links

- **Architecture Index & Search Matrix**: [`docs/architecture/00_INDEX.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/00_INDEX.md)
- **Detailed Project Structure & Code Style Guide**: [`docs/architecture/10_PROJECT_STRUCTURE_AND_CODE_STYLE.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/10_PROJECT_STRUCTURE_AND_CODE_STYLE.md)
- **Clean Architecture Crate Mapping**: [`docs/architecture/02_CLEAN_ARCHITECTURE_RUST.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/02_CLEAN_ARCHITECTURE_RUST.md)
- **Database & UUIDv7 Consistency**: [`docs/architecture/05_DATA_CONSISTENCY_AND_STORAGE.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/05_DATA_CONSISTENCY_AND_STORAGE.md)
- **Valkey Read Cache & Invalidation**: [`docs/architecture/11_VALKEY_READ_CACHE.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/11_VALKEY_READ_CACHE.md)
- **View Scope & Company Switcher (HQ overview / own-branch only)**: [`docs/architecture/12_VIEW_SCOPE_COMPANY_SWITCHER.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/12_VIEW_SCOPE_COMPANY_SWITCHER.md)
- **Ory Kratos Auth & Security**: [`docs/architecture/06_SECURITY_IDENTITY_AUTH.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/architecture/06_SECURITY_IDENTITY_AUTH.md)
- **Programs & Screens Catalogue**: [`docs/programs/00_INDEX.md`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/docs/programs/00_INDEX.md)
