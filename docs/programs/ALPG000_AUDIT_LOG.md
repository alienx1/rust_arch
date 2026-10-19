---
id: "019ca1ed-0000-7000-8003-000000000031"
code: "ALPG000"
system_code: "AL"
module_code: "ALMT"
route_path: "/al/alpg000"
icon: "FileSearchOutlined"
title_en: "Audit Log"
parent_group: "ALPG000_GROUP"
status: "Implemented (Quickwit + Redpanda CSV Export)"
tags:
  - program
  - audit
  - compliance
  - quickwit
  - screen
---

# 🔍 ALPG000 - Audit Log

> [!INFO] **Program Metadata**
> - **Program Code**: `ALPG000`
> - **Title (EN)**: Audit Log
> - **Route Path**: `/al/alpg000`
> - **Icon**: `FileSearchOutlined` (`icondata_ai`)
> - **System**: `AL` (Audit)
> - **Module**: `ALMT` (Audit Maintenance)
> - **Parent Folder**: `ALPG000_GROUP` (Audit)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000031`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for inspecting and searching the complete history of system and business activities (Security & Business Audit Trail) for compliance, incident investigation, and operational transparency. Powered by high-speed full-text search via the **Quickwit OpenSearch-compatible Engine** backed by Object Storage (Garage S3).

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Allow auditors and administrators to query historical events, inspect request/response payloads, and review row-level before/after JSON diffs.

### 2.2 Fields & Schema
- [x] **Search Filters**:
  - Timestamp Range: From – To
  - Actor / User: `user_id`, `username`, `email`
  - Client IP Address & User Agent
  - Target Program: e.g. `DBPG001`, `SUPG004`
  - Event Kind / Action: `api`, `data_change`, `page_view`, `click`, `action`, `event_report_request`, `event_notification`
  - Full-text Search: searches across payloads and diffs
- [x] **Event Detail View**:
  - Event ID & Timestamp (UTC / Local ICT)
  - Correlation ID / Request ID
  - Affected Tables & Record IDs
  - Before / After Snapshot Diff

### 2.3 Business Rules & Actions
- [x] **Immutability**: All audit log entries are strictly append-only / Write Once Read Many (WORM) — they can never be modified or deleted through this screen.
- [x] **Export CSV**: Clicking "Export CSV" in the filter bar sends `POST /api/v1/al/alpg000/exports` (accepting the same filter parameters as the list query, without `page`/`limit`). Requires `READ` permission on `ALPG000` and runs asynchronously in the background via Redpanda without blocking the UI:
  1. `api-master` validates permissions and filter parameters, then publishes a `ReportJobMessage` to the `report.requests` topic (key = job ID) → responds **202** `{job_id}` (or `503` if the broker is unreachable).
  2. `api-report` (worker, consumer group `report-worker`) consumes the message → scrolls all matching events from Quickwit in batches of 5,000 rows → streams a CSV via multipart upload (8 MiB parts, without buffering the whole file in memory) to Object Storage (Garage) at `reports/<job_id>.csv`, capped at 5,000,000 rows.
  3. Job state is tracked in `rp_report_job` (`running` → `succeeded` | `failed`), and the requester is notified via `NotificationService` (publishing to the `notifications` topic) → **notification service** (`NTPG000`) persists the notification in `nt_notification` and pushes it over SSE.
  4. The web UI receives the notification in the topbar bell (across all pages, tabs, and devices signed in as that user) → navigates to "My Reports" (`RPPG000`, `/rp/rppg000`) → previews the first rows (via ranged GET) or downloads the file (`api-report` returns a 5-minute pre-signed URL as JSON so the browser opens it directly rather than following a proxy redirect that would invalidate the `Host` signature).
  - Only the requester can view or download their own report (other users receive `404`).
  - File format: UTF-8 with BOM (so Excel renders Thai characters properly), CRLF line endings, 1 event per row, filename `audit-log-YYYYMMDD-HHMMSS.csv`.
  - Columns: `time_th` (UTC+7), `time_utc`, `kind`, `user_id`, `username`, `email`, `ip`, `method`, `path`, `query`, `status`, `duration_ms`, `program`, `action`, `page`, `element`, `label`, `tables`, `summary`, `changes`, `request_id` — request/response headers and bodies are omitted from the CSV for readability; for `data_change` events, `changes` is formatted on a single line as `table OP row_id: col: old -> new` separated by ` | ` (truncated at 1,500 chars).
  - Values starting with `=`, `+`, `-`, or `@` are prefixed with `'` to prevent spreadsheet formula injection (CSV injection).
  - If object storage fails mid-upload → the job is marked `failed` (multipart upload is aborted so no partial object remains) and a failure notification is sent.
  - Limitations: Excel supports at most 1,048,576 rows per sheet — larger exports should be narrowed by date range or opened with Power Query / a database tool; users offline when the job finishes will see the completed report in `RPPG000` (and in their notification list upon reconnecting); the `rp_report_job` row is created when the worker picks up the message (if the worker is down, the request waits in Redpanda).
- [ ] **Export PDF**: Not implemented yet.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: Search and view audit logs, and request CSV exports
  - `WRITE`: *(Unused — Audit Log is read-only)*
  - `APPROVE`: Approve large-scale audit exports
- **Access Restriction**: Reserved for Auditors, Security Officers, and Super Admins.

---

## 🗄️ 4. Storage Engine
- **Search Engine**: Quickwit 0.8 (Index: `audit-logs-v1`, mapping at `infrastructure/src/services/quickwit/audit_logs_v1_index.json`). The application automatically creates the index if missing (including after a Quickwit restart).
- **Ingestion**: Batched directly to Quickwit's REST ingest endpoint (flushed every 1 second or 200 events) via a non-blocking background queue; if Quickwit is down, events are dropped after brief retries without impacting API latency.
- **Not Yet Implemented**: Dedicated Redpanda `audit.events` ingestion pipeline, cold S3 archival bucket `arch-audit`, PDF export, and `SELECT` query logging.
- **Data Changes (`kind=data_change`)**: The `al_capture_change` PostgreSQL trigger (in `sql/01_ysql.sql`) attached to every table captures `INSERT` / `UPDATE` / `DELETE` operations (ignoring `UPDATE`s that only touch `row_version`) along with before/after row snapshots (redacting password/secret/token columns) into the `al_data_change` outbox table within the same database transaction. `spawn_data_change_shipper` groups rows sharing the same API `request_id` into **a single audit event** (`changes` = list of mutated rows with `table`, `op`, `row_id`, `columns`, `old`, `new`; `detail` = per-table summary), ships them to Quickwit every 2 seconds, and deletes them from the outbox. The actor is read from PostgreSQL session settings `app.actor_id` / `app.actor_name` / `app.request_id` populated by the `audit_trail` middleware (`infrastructure::with_actor` + `track_actor`); `request_id` matches the `event_id` of the corresponding `api` audit event. Database writes without a session actor (seeds, migrations, background workers) are not captured by the trigger.
- **Dev Note**: Quickwit data is stored on PVC `arch-logging-data` (`local-path`) — data survives pod restarts, but is removed if the PVC or cluster is deleted.
- **Configuration**: `QUICKWIT_URL` (default `http://localhost:7280`), `AUDIT_LOG_ENABLED` (default `true`).

### 4.1 Recorded Event Kinds (`AuditEvent`, `application/core/src/traits/audit_log.rs`)
| Kind | Source | Captured Content |
|---|---|---|
| `api` | Server middleware for every `/api/v1/**` request | Actor (`user_id` = Kratos identity ID, `username`, `email`, `session`, `ip`, `user_agent`, `language`), target (`method`, `path`, `query`, `program`, `action`), HTTP `status`, `duration_ms`, headers, and request/response bodies |
| `page_view` | Web client | Route path opened by the user |
| `click` | Web client | Button or link clicked (element tag + button label + current page); never records text typed into input fields |
| `action` | Web client | Completed client workflow action, e.g. `DBPG004.save`, `SUPG003.matrix.save`, `SUPG004.recovery_link` |

- **Redaction**: Headers such as `cookie`, `authorization`, `x-api-key`, etc., and any body/query fields whose names contain `password`, `secret`, `token`, `cookie`, `api_key`, or `credential` are replaced with `***`.
- **Bodies**: CBOR payloads are decoded to JSON (with UUIDs formatted as hyphenated strings) and stored up to 16 KiB; streaming bodies or payloads exceeding 256 KiB are omitted (recording only their byte size).
- **Excluded Paths**: `/api/v1/audit/**`, `ALPG000`'s own read queries, i18n dictionary fetches, and `OPTIONS` preflight requests.

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/al/alpg000/` — `/al/alpg000` (filterable data table: date range, kind, HTTP status `2xx`–`5xx`, user, method, program, path, full-text query) and `/al/alpg000/log/{id}` (event detail + formatted request/response JSON & row diffs); client-side activity tracker in `src/audit/` (page views, clicks, `track_action`) flushes batches every ~3 seconds.
- **Backend API** (`/api/v1/al/alpg000`, requires `READ` permission): `GET /logs`, `GET /logs/{id}`, `GET /status`; any authenticated user can submit their own client activity events to `POST /api/v1/audit/events` (session-authenticated). Returns `503 AUDIT_UNAVAILABLE` if Quickwit is unreachable.
- **Application Layer**: `application/master/src/features/al/alpg000/`; port `AuditLogPort` in `application/core`.
- **Infrastructure Layer**: `infrastructure/src/services/quickwit/` (`QuickwitAuditLog`: queue + batch flusher + query builder); middleware `infrastructure/src/middleware/audit/audit_trail.rs`.
- **Search Behavior**: `path` searches by token/prefix (e.g. `dbpg004`), not arbitrary mid-word substrings; full-text queries are tokenized by Quickwit's default tokenizer.
- **Tests**: `alpg000_test.rs`, `alpg000_router_test.rs`, `audit_trail_test.rs`, `infrastructure/tests/quickwit_audit_query_test.rs` (includes `--ignored` live Quickwit integration tests).

## Redpanda Stream Events (`event_report_request`, `event_notification`)

The "Kind" filter includes two additional event types that query Quickwit indexes `events-report-requests-v1` and `events-notifications-v1` (ingested directly by Quickwit's Kafka source, configured via `make event-indexes` — see `NTPG000_NOTIFICATIONS.md`) instead of `audit-logs-v1`:

- Selected one kind at a time (not combined with other kinds); supports date range, user (requester / recipient), and full-text search filters — HTTP method, status, program, and path filters do not apply to stream events.
- Clicking an event row opens its full detail view as usual (`GET /logs/{id}` looks up the job ID or notification ID).
- CSV export is not supported for these stream event kinds (export requests with these kinds return `400`).
- Implementation: `infrastructure/src/services/quickwit/event_stream_query.rs`, tested in `infrastructure/tests/event_stream_query_test.rs`.

## Date Range Picker

The time filter uses the shared `web_package::ui::component::DateRangePicker` popover button:

- **Left pane**: quick presets — Today, Yesterday, Last 7 Days, Last 30 Days, This Month, Last Month, All Time.
- **Right pane**: monthly calendar (displays Buddhist Era year when viewing in Thai). Click the start date, hover to preview the range, and click the end date to apply immediately; click outside or press `Esc` to close; expands full-width on mobile viewports.
- Default selection = **Today**; "All Time" = no timestamp filter.
- Dates are interpreted in the browser's local timezone: `from` = `00:00` of the start date, `to` = `00:00` of the **day after the end date** (exclusive), covering the entire end date — e.g. Oct 12 in UTC+7 sends `from=2026-10-11T17:00:00Z&to=2026-10-12T17:00:00Z`.
- "Export CSV" uses the exact same selected date range.
