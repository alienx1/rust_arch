---
id: "019ca1ed-0000-7000-8003-000000000040"
code: "RPPG000"
system_code: "AL"
module_code: "ALMT"
route_path: "/rp/rppg000"
icon: "FileDoneOutlined"
title_en: "My Reports"
parent_group: "ALPG000_GROUP"
status: "Implemented (api-report + S3 + notifications)"
tags:
  - program
  - report
  - async-job
  - storage
  - screen
---

# 📑 RPPG000 — My Reports

> [!INFO] **Program Metadata**
> - **Program Code**: `RPPG000`
> - **Title (EN)**: My Reports
> - **Route Path**: `/rp/rppg000`
> - **Icon**: `FileDoneOutlined` (`icondata_ai`)
> - **System**: `AL` (Audit)
> - **Module**: `ALMT` (Audit Maintenance)
> - **Parent Folder**: `ALPG000_GROUP` (Audit)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000040`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

Central hub for generated report files and background export job history belonging to the currently signed-in user. When a user triggers a data export from a screen (such as exporting Audit Logs from [[ALPG000_AUDIT_LOG|ALPG000]]), the request is dispatched via Redpanda to a background worker that streams the output into Garage S3. Once complete, the user receives a real-time notification via [[NTPG000_NOTIFICATIONS|NTPG000 (Notifications)]] and can preview or download the file on this screen.

---

## 🔄 End-to-End Architecture Flow

```text
1. User clicks Export CSV in ALPG000 ────────▶ api-master publishes ReportJobMessage to Redpanda topic `report.requests`
                                                                    │
                                                                    ▼
                                           api-report worker (consumer group `report-worker`)
                                            1. Inserts/updates job status in rp_report_job ('queued' -> 'running')
                                            2. Streams matching records from Quickwit / DB into CSV (with UTF-8 BOM)
                                            3. Uploads via CsvObjectWriter to Garage S3 (`reports/{id}.csv`)
                                            4. Updates status to 'succeeded' with row_count and size_bytes
                                            5. Publishes completion event to topic `notifications`
                                                                    │
                                                                    ▼
                                           api-notification pushes SSE event to the user's browser
                                                                    │
                                                                    ▼
                                           User clicks the notification or opens /rp/rppg000
                                            • Views status of all requested report jobs
                                            • Previews the first rows in a modal dialog
                                            • Requests a short-lived Pre-signed S3 URL to download the file
```

---

## 🗄️ Database Schema & Object Storage

### 1. Table `rp_report_job` (`sql/01_ysql.sql`)
Stores job request metadata and execution results:
- `id` (UUIDv7, PK): Unique job identifier
- `report_kind` (`varchar(50)`): Report type, e.g. `audit_log_csv`
- `requested_by` (`varchar(255)`): Kratos username of the requester
- `filter` (`jsonb`): Filter parameters captured when the export was requested
- `status` (`varchar(20)`): Job state (`queued`, `running`, `succeeded`, `failed`)
- `row_count` (`bigint`): Total number of data rows written to the file
- `size_bytes` (`bigint`): Total file size in bytes
- `file_name` (`varchar(255)`): Suggested download filename (e.g. `audit-log-20261012-152000.csv`)
- `object_key` (`varchar(500)`): Object path in Garage S3 (e.g. `reports/{job_id}.csv`)
- `error` (`text`): Error description if the job failed
- `requested_at` / `started_at` / `finished_at` (`timestamptz`)

### 2. Object Storage (Garage S3)
- Bucket: `rust-arch-bucket`
- Key pattern: `reports/{job_id}.csv`
- Streamed via `CsvObjectWriter` using multipart upload chunks so large exports keep a constant memory footprint.

---

## 🔌 API Endpoints (`api-report`, Port 8001)

All endpoints are mounted under `/api/v1/reports` (requires a valid Kratos session and restricts visibility strictly to the user's own jobs):

| Method | Path | Summary | Details |
|---|---|---|---|
| `GET` | `/api/v1/reports/jobs` | List current user's report jobs | Ordered newest first; supports JSON and CBOR wire formats |
| `GET` | `/api/v1/reports/jobs/{id}` | Get single job status by ID | Returns `ReportJobDto` (returns `404` if owned by another user) |
| `GET` | `/api/v1/reports/jobs/{id}/preview?rows=20` | Preview initial rows | Streams the header and initial rows from S3 via ranged GET and parses them into a preview table |
| `GET` | `/api/v1/reports/jobs/{id}/download` | Request download link | Returns JSON `{ "url": "..." }` containing a short-lived Pre-signed S3 URL |

---

## 💻 User Interface & Clean UI

- **Route Path**: `/rp/rppg000`
- **Component Shell**: Page header with usage guidance, Refresh button, and `DataTable`.
- **Report Table (`ReportJobTable`)**:
  - **Requested At**: Timestamp formatted in Thailand timezone (UTC+7)
  - **Report Kind**: Report type label and icon
  - **Status**:
    - ⏳ `queued` (Queued)
    - 🔄 `running` (Running)
    - ✅ `succeeded` (Completed)
    - ❌ `failed` (Failed, with error reason)
  - **Rows / Size**: Row count and human-readable file size (`KB`, `MB`)
  - **Actions**:
    - **"Preview"** button (opens `ReportPreviewDialog` displaying columns and initial rows)
    - **"Download"** button (fetches and opens the Pre-signed S3 URL)
- **Process Monitor Integration**: If a background job fails or gets stuck, administrators can inspect and reprocess it in [[ALPG001_PROCESS_MONITOR|ALPG001 (Process Monitor)]].

---

## 🧱 Code Layout

- **Domain Entity**: `domain/src/entity/rp_report_job.rs`
- **Application Layer**: `application/report/src/features/rppg000/`
  - `commands/`: `list_jobs.rs`, `get_job.rs`, `download_job.rs`, `preview_job.rs`, `run_report_job.rs`, `csv_object_writer.rs`, `audit_csv_rows.rs`
  - `queries/`: `db_rppg000_queries.rs`
  - `models/`: DTOs, `job_status` constants, `DOWNLOAD_LINK_SECS`, `UPLOAD_PART_BYTES`
- **API Delivery**: `handler/api/report/src/routers/rppg000_router.rs`
- **Frontend SPA**: `handler/web/application/master/src/pages/rp/rppg000/`
  - `components/`: `report_job_table.rs`, `report_preview_dialog.rs`
  - `providers/`: `rppg000_context.rs`
  - `services/`: `rppg000_service.rs`
  - `screens/`: `rppg000_screen.rs`
- **Unit & Integration Tests**: `application/report/tests/rppg000_test.rs`

---

## 🔗 Cross-References
- [[ALPG000_AUDIT_LOG|ALPG000 — Audit Log]]: Origin of CSV export job requests
- [[ALPG001_PROCESS_MONITOR|ALPG001 — Process Monitor]]: Admin screen for monitoring and reprocessing background jobs
- [[NTPG000_NOTIFICATIONS|NTPG000 — Notifications]]: Real-time SSE notification delivery when reports finish
- [[07_EVENT_STREAMING_OBSERVABILITY|Chapter 7: Event Streaming & Observability]]: Redpanda `report.requests` architecture and Quickwit indexing
