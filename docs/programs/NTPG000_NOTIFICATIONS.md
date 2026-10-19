# 🔔 NTPG000 - Notifications

Central notification service used across all backend services to alert users about asynchronous events (completed reports, pending approvals, etc.), surfaced via the topbar bell icon on every page as well as the dedicated `/nt/ntpg000` screen.

## Architecture Flow

```text
Any backend service ──NotificationService.notify()──▶ Redpanda topic `notifications` (key = recipient)
                                                            │
                                                            ▼
                                            api-notification (consumer group `notification-service`)
                                             1. Persist into nt_notification (id = UUIDv7 → duplicate deliveries are ignored)
                                             2. Push over SSE to all active sessions of the recipient
                                                            │
                                                            ▼
                                      Web UI: Topbar bell (unread count + 10 latest items) / NTPG000 page
```

- **Producers** call the `NotificationService` port (`application-core`) backed by `RedpandaNotificationPublisher` (`infrastructure`), with zero coupling to SSE or the notification database table: `notify(username, Notification { id, kind, title, body, data, link, created_at })`.
- **Recipients** are addressed by Kratos username (every browser tab and device signed in as that user receives the event simultaneously; other users never see it).
- `link` = internal route path opened when clicking the notification row (must start with `/`; external links are stripped to prevent phishing).
- Read state (`read_at`) is stored server-side and broadcast via an SSE `changed` event so all open tabs refresh immediately when a notification is marked read in any tab.
- **Localization (i18n)**: Producers store fallback `title`/`body` strings and pass structured interpolation parameters inside `data` (e.g. `report_ready` → `{job_id, file_name, rows}`, `report_failed` → `{job_id, reason}`). The web UI looks up `ntpg000.kind.<kind>.title` and `.body` in the active language dictionary (`db_label`, seeded in `sql/83_i18n_ntpg000_kinds_sound.sql`) and interpolates `{placeholder}` values from `data` — switching UI language updates notification text immediately. Unknown kinds or older records missing `data` fields gracefully fall back to the stored `title`/`body`. To add a **new notification kind**, seed those two i18n keys (`th`/`en`) and include the placeholders in `data`.
- **Notification Sound**: When a new notification arrives over SSE (`notification` event only — never triggered on `changed` or initial page load), the browser synthesizes a short two-note chime via the Web Audio API (no external audio file needed). Enabled by default and toggleable via the speaker icon in the bell popover header (persisted in `localStorage` under `notification_sound`). Because browsers block Web Audio playback until the user has interacted with the page at least once (click/keypress), notifications arriving before first interaction are silent (the unread badge still updates); multiple open tabs each chime once.
- **300-Item Cap per User**: Whenever a new notification is ingested, the service prunes that user's oldest notifications beyond the latest 300 (`MAX_NOTIFICATIONS_PER_USER`, regardless of read status) — other users are unaffected.
- Users who are offline when a notification is published will see it the next time they open the app (loaded from the server when the SSE stream connects).
- **Marking as Read**: A notification is marked read **only when clicking that row** (either in the bell popover or on the `NTPG000` page) or clicking "Mark All as Read" — simply opening the bell popover or visiting `/nt/ntpg000` does not mark items as read.

## API (`api-notification`, Port 8002, Kratos Session Required)

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/notifications/items?limit=` | List current user's notifications (newest first, default 20, max 300) + total `unread` count |
| `POST` | `/api/v1/notifications/{id}/read` | Mark a single notification as read (returns `404` if not found or owned by another user) |
| `POST` | `/api/v1/notifications/read-all` | Mark all of the current user's notifications as read |
| `GET` | `/api/v1/notifications/stream` | SSE stream: `ready`, `notification` (new item), `changed` (read state updated → reload list) |

## Database & Code Layout

- Table: `nt_notification` (`sql/01_ysql.sql`), entity: `domain/src/entity/nt_notification.rs`
- Application crate: `application/notification` — feature `ntpg000` (ingest / list / mark read) with unit tests in `application/notification/tests/`
- API binary crate: `handler/api/notification` — Redpanda consumer + REST + SSE (`make run-notification` / `make notification`)
- Web UI: `handler/web/application/master/src/pages/nt/ntpg000/` (topbar bell is mounted in `MasterLayout` via `MainLayout`'s `topbar_actions`)
- Trunk dev proxy: `/api/v1/notifications/ → 8002` (must be listed before `/api/v1/`)

## Multi-Instance Deployment

Because the SSE hub lives in each instance's memory and every replica must receive every message to push to its connected browser sessions, each replica must configure a **unique** `NOTIFICATION_CONSUMER_GROUP` (database inserts remain safe across replicas because duplicate UUIDv7 primary keys are ignored via `ON CONFLICT DO NOTHING`).

## Adding a New Notification Kind

Define a new `kind` constant in `notification_kind`, supply fallback `title`/`body` strings, populate `data` with kind-specific parameters, set `link` to the target relative route path, and call `notify`. No changes to `api-notification` are required.

## Limitations

- Producer publishing is fire-and-forget: if the Redpanda broker is down, the error is logged without failing the originating business operation.
- Time-based archival beyond the 300-item per-user cap is not yet configured.
- The Redpanda topic must exist prior to publishing: run `make topics`.

## Event Indexing in Quickwit

All messages published to Redpanda topics `report.requests` and `notifications` are automatically indexed into Quickwit using Quickwit's native Kafka source (zero extra application code, consumer groups `quickwit-events-*`, 90-day retention):

| Index | Topic | Timestamp Field |
|---|---|---|
| `events-report-requests-v1` | `report.requests` | `requested_at` |
| `events-notifications-v1` | `notifications` | `created_at` |

- Create indexes and sources: `make event-indexes` (idempotent; automatically called by `make topics`) — index configs live under `kube/arch/applications/arch_logging/indexes/`
- Direct search query: `curl 'localhost:7280/api/v1/events-notifications-v1/search?query=recipient:<user_id>'`
- Quickwit storage is backed by a PVC so indexed events persist across pod restarts.
