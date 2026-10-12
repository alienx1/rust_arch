-- ALPG000: Redpanda events (report requests, notifications) kept in Quickwit.
-- Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'alpg000', 'filter.kind_event_report_request', 'Event: คำขอรายงาน (Redpanda)'),
  ('en', 'alpg000', 'filter.kind_event_report_request', 'Event: report requests (Redpanda)'),
  ('th', 'alpg000', 'filter.kind_event_notification', 'Event: การแจ้งเตือน (Redpanda)'),
  ('en', 'alpg000', 'filter.kind_event_notification', 'Event: notifications (Redpanda)'),
  ('th', 'alpg000', 'kind.event_report_request', 'Event: คำขอรายงาน'),
  ('en', 'alpg000', 'kind.event_report_request', 'Event: report request'),
  ('th', 'alpg000', 'kind.event_notification', 'Event: แจ้งเตือน'),
  ('en', 'alpg000', 'kind.event_notification', 'Event: notification')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
