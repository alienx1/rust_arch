# ALPG001 — Process Monitor

Inspect whether **background jobs dispatched through Redpanda** succeeded or failed, and trigger **reprocessing** when needed.
Currently tracks one background process type: CSV report generation requested from `ALPG000` (`report_kind = audit_log_csv`, stored in `rp_report_job`).

- **Web Route**: `/al/alpg001` (Menu: Audit → Process Monitor) • Program `ALPG001` in `sql/52_program_catalogue.sql`
- **API**: `/api/v1/al/alpg001` (`api-master`) — viewing requires `READ`; triggering reprocess requires **`WRITE`** on `ALPG001`
- **Code**: `application/master/src/features/al/alpg001`, `handler/api/master/src/routers/alpg001_router.rs`, `handler/web/application/master/src/pages/al/alpg001`

## Process States

| Status | Meaning |
|---|---|
| `queued` | Waiting for a worker to pick up the job (after being re-queued) |
| `running` | Worker is actively generating the output |
| `succeeded` | Completed successfully (file stored in Garage S3) |
| `failed` | Failed — failure reason is shown in the "Result" column |

A `queued` or `running` job that has made no progress for over **15 minutes** is flagged as **stuck** (worker crashed or message lost) — it displays a "Stuck" badge and becomes eligible for reprocessing.

## User Interface

- **Filters**: Requested date range (`DateRangePicker`, defaults to "All Time"), status filter chips with live counts, and requester search.
- **Columns**: Requested At, Process (+ Job ID), Requested By, Status, Result (row count / file size / filename or error message), Duration, Actions.
- While any job on the page is `queued` or `running`, the table automatically polls for updates every 4 seconds.

## Reprocess Workflow

`POST /api/v1/al/alpg001/processes/{id}/reprocess` → `202 Accepted`

1. **Eligibility check**: `succeeded` and `failed` jobs can always be reprocessed (useful for regenerating a file if it was removed from Garage S3); `queued` and `running` jobs can only be reprocessed if they are **stuck** (> 15 minutes); otherwise returns **409 `PROCESS_NOT_REPROCESSABLE`**.
2. **Atomic conditional `UPDATE`**: resets the job row to `queued`, clears `error`, and sets `started_at = now` in a single conditional statement (preventing race conditions if two admins click reprocess simultaneously).
3. **Republish event**: sends the original `ReportJobMessage` (reconstructed from `rp_report_job.filter`) back to the `report.requests` topic with `reprocess: true`.
   — If the Redpanda broker is unreachable, reverts the database row to its prior state and returns **503 `PROCESS_QUEUE_UNAVAILABLE`**.
4. **Worker execution**: `api-report` receives the message; when `reprocess: true`, it executes only if the row is currently `queued` or `running` (so duplicate deliveries after completion are safely ignored), overwrites `reports/<id>.csv` in Garage S3, updates the job row, and sends a completion notification to the original requester.

Note: The original requester receives a new `report_ready` or `report_failed` notification when reprocessing finishes.

## Scope & Limitations

- Only displays jobs that a worker has **already picked up** (since the `rp_report_job` row is created by the worker upon consuming the message). Requests waiting in the Redpanda topic while all workers are down can be inspected in `ALPG000` under `Event: Report Request`.
- Notification delivery events (`notifications` topic) are not tracked in this table.
