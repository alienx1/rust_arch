# Guide: Add a New Program (Screen + API) End-to-End

Reference implementations you can inspect in the codebase: **ALPG001 Process Monitor** (table read + action button, with complete tests) and **DBPG005 Geography** (full CRUD).
Suppose we are adding program `XXPGNNN` under system `XX` — replace those placeholders with your actual identifiers throughout this guide.

> Invariant rules live in [`AGENTS.md`](../../AGENTS.md) (dependency layers, strict ban on generic filenames such as `service.rs`/`handler.rs`/`utils.rs`, tests strictly inside `tests/`, UUIDv7, Axum 0.8 `{param}` syntax).

## 0. Decisions Before Coding

| Item | Value to Choose |
|---|---|
| Program Code | `XXPGNNN` — `XX` = 2-letter system code, `PG` = page, `NNN` = sequence number (uppercase) |
| Web Path | `/xx/xxpgnnn` (lowercase — the menu uses this as the primary route for the program) |
| API Path | `/api/v1/xx/xxpgnnn/...` — **must contain the `xxpgnnn` segment** because the audit middleware resolves the request's target program from this segment (`infrastructure/src/middleware/audit/audit_target.rs`) |
| Permissions | `READ` (view), `WRITE` (create/update/delete/trigger action), `APPROVE` (approve) — enforced in the router via `require(&access, &user, "XXPGNNN", PermissionAction::…)` |
| Data Scope | If actions are scoped to an organization/company/branch/department, use `require_in` + `ScopeTarget` (see `DBPG003` / `access_scope_test.rs`) |
| Menu Placement | Placed under a parent folder (e.g. `ALPG000_GROUP`) or top-level; if the screen belongs to the "current user" (My Reports, Notifications), register it in the user profile dropdown — see `USER_MENU_PROGRAMS` in `layouts/master_navigation.rs` |

## 1. Database

1. Add the table to [`sql/01_ysql.sql`](../../sql/README.md) — named `xx_<name>`, PK `uuid` (generated as **UUIDv7** in Rust via `Uuid::now_v7()`), with `row_version bigint NOT NULL DEFAULT 1` (optimistic locking) if mutable.
   - Place the table **before** the trailing `DO $$ … al_capture_change …` block at the bottom of `01_ysql.sql` so it automatically receives the before/after audit capture trigger.
   - Tables added later via migration files must explicitly create the `al_capture_change` trigger (see [`sql/README.md`](../../sql/README.md), "Changing the Schema").
2. If the table is required for the app to boot and its absence should force a database recreation hint, add the table name to the guard in `kube/arch/utils/db-seed.sh`.
3. Generate Sea-ORM entities: ensure the table exists in the database (`make seed` or `make bootstrap`), then run `make gen-entities` → generates `domain/src/entity/xx_<name>.rs` + updates `mod.rs`/`prelude.rs`.
   - This command **regenerates all entities**: always inspect `git diff` to ensure custom edits in existing entity files are not overwritten.

## 2. Application Layer — `application/master/src/features/xx/xxpgnnn/`

```text
features/xx/xxpgnnn/
├── mod.rs                      # Re-exports + program doc comment (code, API, data source)
├── errors/mod.rs               # Enum XxpgnnnError (thiserror): Validation / NotFound / Conflict / Unavailable / Internal
├── models/                     # DTOs + filter query structs + (re-exported entities as *Model)
├── commands/
│   ├── mod.rs                  # Trait XxpgnnnCommands  ← routers call exclusively through this trait
│   ├── xxpgnnn_executor.rs     # Struct …CommandExecutor implementing the trait (thin facade delegating to <action>.rs)
│   └── <action>_<target>.rs    # Per-action business logic (list_processes.rs, reprocess_process.rs …)
└── queries/
    ├── mod.rs                  # Trait XxpgnnnQueries
    └── db_xxpgnnn_queries.rs   # Direct Sea-ORM queries (no redundant repository layer)
```

- Register the module in `features/xx/mod.rs` (`pub mod xxpgnnn;`) — and re-export in `features/mod.rs` if needed.
- Responsibilities: validate and normalize inputs in `commands` (e.g. `filter_rules.rs`), keep pure data access in `queries`, and **never import `handler/*` or concrete infrastructure drivers directly** — depend on ports from `application-core` (e.g. `EventService`, `CacheService`).
- Design for testability: the executor accepts `Arc<dyn …Queries>` (+ optional ports via `.with_xxx(...)`) so unit tests can inject in-memory mocks.
- If a query reads data that is "read frequently, modified rarely", wrap it with the read cache (see Section 6).
- **Optimistic locking (update commands)**: `Update…Command` must carry `row_version: i64` (the version loaded by the form) and save through the shared helper — never compare versions or increment `+ 1` manually:
  ```rust
  update_versioned(&txn, am, (db_x::Column::Id, db_x::Column::RowVersion), id, cmd.row_version).await?;
  ```
  (`crate::features::optimistic_update`). Set only the modified columns on `am`; the helper executes `UPDATE … SET …, row_version = row_version + 1 WHERE id = ? AND row_version = ?`. If no row matches, it returns `DbErr::RecordNotUpdated`. Ensure your feature error's `From<DbErr>` maps `DbErr::RecordNotUpdated => Self::VersionConflict` (already standard across existing programs), and the router maps `VersionConflict` to **HTTP 412 `VERSION_CONFLICT`**. Always update the parent row before child rows within the same transaction.

## 3. API Layer — `handler/api/master/`

1. `src/routers/xxpgnnn_router.rs` — model after `alpg001_router.rs`:
   - Define `pub const XXPGNNN_PATH: &str = "/xx/xxpgnnn";` and `const PROGRAM: &str = "XXPGNNN";`.
   - Handlers extract `State(commands): State<Arc<dyn XxpgnnnCommands>>` + `State(access): State<Arc<dyn AccessControl>>` + `Extension(user)` + `wire: WireFormat` → check `require(...)` first → return responses via `wire.respond(...)` (automatically negotiates JSON/CBOR).
   - Implement `IntoResponse` for the feature's `ApiError` to map domain errors to HTTP status codes (`400`/`404`/`409`/`412`/`503`/`500`) with `{success, code, message}`.
   - Expose `xxpgnnn_routes()` and `xxpgnnn_routes_with_state(state)` (the latter for router tests).
   - Use Axum 0.8 path parameter syntax `/{id}`, never `/:id`.
2. Register the router: `routers/mod.rs` (`pub mod` + `pub use`) and `routers/base_router.rs` (**must `.nest` in two places**: inside `api_router` and inside the root router under `/api/v1`).
3. Wiring (**not** `connection.rs`): create `src/wiring/xxpgnnn.rs` with `pub fn register(cx: &WiringContext, services: &mut ServicesBuilder)` that builds the executor from the shared context (`cx.db`, `cx.caches`, `cx.kratos`, `cx.broker` …; chain `.with_cache(cx.caches.clone())` if cached; take another feature with `services.require::<dyn OtherCommands>()`) and registers it with `services.insert::<dyn XxpgnnnCommands>(Arc::new(..))`. Add `pub mod xxpgnnn;` and one line `xxpgnnn::register(cx, &mut s);` to `wiring/mod.rs` (after any feature it builds on), and one `assert!(s.has::<dyn XxpgnnnCommands>())` line to `tests/wiring_test.rs`. The router's `State<Arc<dyn XxpgnnnCommands>>` then resolves by type (one blanket `FromRef<AppConnection> for Arc<T>` in `connection.rs`: no field and no per-feature `FromRef`, so two branches adding programs do not collide there).
4. Data mutations: pass `actor.user_id` as `change_by`; the database audit trigger automatically captures before/after snapshots when the API sets the session actor (`infrastructure::track_actor`).
5. If this program mutates data cached under any namespace → add an invalidation rule in `src/cache_policy.rs` (see Section 6).

> **Notices after a save are automatic**: every `POST/PUT/PATCH/DELETE` on `/api/v1/<system>/<xxpgnnn>/…` notifies the user who made it (`data_created|updated|deleted|failed`) through the NTPG000 SSE stream (toast on every page), with no per-page code; a success is live only (not stored, not indexed), a failure is kept in the bell. A new program needs no notification code; if it reports on its own (like reports), add its code to `QUIET_PROGRAMS` in `infrastructure/src/middleware/notice/change_target.rs`. See [`NTPG000_NOTIFICATIONS.md`](../programs/NTPG000_NOTIFICATIONS.md).

## 4. Tests (Never Inline Inside `src/`)

| Location | What to Test |
|---|---|
| `application/master/tests/xxpgnnn_test.rs` | Validation/normalization rules, executor behavior against mock queries, error cases |
| `handler/api/master/tests/xxpgnnn_router_test.rs` | HTTP status codes: `200`/`202`, `400`, `403` (missing permission / READ-only), `404`, `409`, `503` using `…_routes_with_state` + `MockAccess` |

Run: `cargo test -p application-master --test xxpgnnn_test` and `cargo test -p api-master --test xxpgnnn_router_test`
(Complete reference examples: `alpg001_test.rs`, `alpg001_router_test.rs`).

## 5. Seed Data — Menu Catalogue + Permissions + i18n Labels

1. **`sql/52_program_catalogue.sql`**: insert a row into `db_program` (fixed UUID in the `019ca1ed-0000-7000-8003-…` range, `code`, `path`, `icon`, `sort_order`, `program_id` = parent folder) and Thai/English labels in `db_program_label`.
   - The `icon` name must exist in `handler/web/package/src/layout/nav_icons.rs` (unknown icons render as a dot); add new icon mappings there if needed.
2. **Permissions**: no manual SQL needed for `ADMIN` — `sql/53_admin_seed.sql` automatically grants `READ`/`WRITE`/`APPROVE` on **every active program** to the `ADMIN` group on each run; other groups can be configured in the `SUPG003` Permission Matrix UI.
3. **UI translations**: create a new file `sql/NN_i18n_xxpgnnn.sql` (`NN` = next available number, idempotent via `ON CONFLICT DO NOTHING`, UUIDv7 IDs — copy the `INSERT … SELECT` pattern from `sql/80_i18n_alpg001.sql`) defining `xxpgnnn.*` keys for both `th` and `en`.
4. Run `make seed` and verify: `SELECT code FROM arch.db_program WHERE code='XXPGNNN'`.

## 6. Read Cache & Invalidation (When Applicable)

- Frequently read / rarely modified / shared across users → wrap reads in the executor: `caches.ns(cache_ns::REF).through_key(key, cache_ttl::REF, || …)` (see `dbpg005_executor.rs`).
- Frequently changing / user-specific / strictly real-time → **do not cache**.
- If the program writes data that invalidates any cached namespace → add an entry in `handler/api/master/src/cache_policy.rs` (verified by `cache_policy_test.rs`).
- Full details: [`docs/architecture/11_VALKEY_READ_CACHE.md`](../architecture/11_VALKEY_READ_CACHE.md).

## 7. Web Frontend — `handler/web/application/master/src/pages/xx/xxpgnnn/`

```text
pages/xx/xxpgnnn/
├── mod.rs                           # pub use screens::XxpgnnnScreen as XxpgnnnPage;
├── models/xxpgnnn_dto.rs            # DTOs (Clone, Debug, PartialEq, Serialize, Deserialize)
├── services/xxpgnnn_service.rs      # HTTP calls (HttpClient) — maps 403/409/503 to user messages
├── providers/xxpgnnn_context.rs     # Reactive context (Leptos signals) + use_xxpgnnn() hook
├── components/<widget>.rs           # Presentational UI widgets isolated to this page
└── screens/xxpgnnn_screen.rs        # Page assembly: PageContainer/PageHeader + provide_…_context()
```

- **Always use `web_package::ui::component::DataTable` for list tables** (search, refresh, filter chips, pagination) — never hand-roll a custom `Card` + `Table` shell.
- Register the page: `pages/xx/mod.rs`, `pages/mod.rs` (`pub use …Page`), and `routes/master_routes.rs` (`use` + `<Route path=path!("xx/xxpgnnn") view=XxpgnnnPage />`).
- Localize all user-facing strings with `t_or("xxpgnnn.key", "Fallback text")` matching the keys in your i18n seed file.
- Permission-gated buttons: hide or disable based on `use_access()` (populated from `/api/v1/access/me`), and **always use confirmation buttons (`ConfirmButton`, `SaveConfirmButton`, `DeleteConfirmButton`) for data-mutating actions**. Note that **the API always re-verifies permissions** — never rely solely on client-side checks.
- Use `DateRangePicker` for date ranges and `dismiss_on_outside` for dropdowns (do not use a full-screen transparent backdrop inside the topbar, because `backdrop-blur` on the topbar traps `fixed` positioning to the topbar bounds).
- Styling: use theme color tokens (`bg-primary`, `text-muted-foreground`, `border-border`, …) so dark mode works seamlessly.

## 8. Documentation

- Create `docs/programs/XXPGNNN_<NAME>.md` (purpose, API endpoints, status, limitations) and add a row to `docs/programs/00_INDEX.md`.

## 9. Pre-Submission Checklist

- [ ] `make seed` succeeds and the program appears in the sidebar menu (when signed in as `admin`)
- [ ] `cargo test --workspace --exclude master-app` passes with zero new warnings
- [ ] `cargo check -p master-app --target wasm32-unknown-unknown` passes
- [ ] Tested in a real browser: loading/empty/error states, permission enforcement (using `test_abac` with READ-only access), Thai/English switching, dark mode, and narrow viewport
- [ ] Verified in `ALPG000` (Audit Log) that API calls and data changes for the new program are recorded
- [ ] Added the program specification under `docs/programs/`

## 10. Common Pitfalls Encountered in This Project

| Symptom | Cause / Fix |
|---|---|
| Menu item does not appear | Forgot to run `make seed`; user's group lacks `READ` permission; or program has `active=false` (sidebar menu is cached for 60s — run `make cache-flush`) |
| API returns 404 despite router registration | Forgot the second `.nest` in `base_router.rs` under `/api/v1…`; requests via Trunk proxy must start with `/api/v1/` |
| Sub-router `/` route is misinterpreted by proxy | Use `/items` instead of `/` (encountered in `api-notification`) |
| `bg-primary/10` or `border-border/60` renders no color | Theme colors use CSS variables `var(--…)` where Tailwind 3 alpha modifiers do not work → use `bg-[color-mix(in_oklch,var(--primary)_12%,transparent)]` |
| Dynamically formatted Tailwind classes (`format!("w-{n}")`) have no effect | Tailwind scans source files statically for complete class names: write full literals (`"w-4"`, `"w-8"`) or add them to `safelist` in `tailwind.config.js` |
| Web UI still shows old code | Trunk is still compiling the WASM bundle or the browser cached it — wait for build completion and hard reload (`Ctrl+Shift+R`) |
| Adding a `web-sys` API fails to compile | Enable the corresponding `web-sys` feature flag in `Cargo.toml` of `handler/web/package` and/or `handler/web/application/master` |
| `row_version` mismatch (`412 VERSION_CONFLICT`) | Edit forms must send back the latest `row_version` loaded from the API; commands must save via `update_versioned` (Section 2) and map `DbErr::RecordNotUpdated` to `VersionConflict` |
