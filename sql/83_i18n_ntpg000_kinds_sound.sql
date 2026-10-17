-- Notification texts per kind (filled with the values in the notification's `data`) and the sound
-- switch of the bell. Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'ntpg000', 'kind.report_ready.title', 'รายงานพร้อมให้ดาวน์โหลด'),
  ('en', 'ntpg000', 'kind.report_ready.title', 'Report ready to download'),
  ('th', 'ntpg000', 'kind.report_ready.body', '{file_name} ({rows} แถว)'),
  ('en', 'ntpg000', 'kind.report_ready.body', '{file_name} ({rows} rows)'),
  ('th', 'ntpg000', 'kind.report_failed.title', 'สร้างรายงานไม่สำเร็จ'),
  ('en', 'ntpg000', 'kind.report_failed.title', 'Report could not be created'),
  ('th', 'ntpg000', 'kind.report_failed.body', '{reason}'),
  ('en', 'ntpg000', 'kind.report_failed.body', '{reason}'),
  ('th', 'ntpg000', 'sound.on', 'เปิดเสียงแจ้งเตือนอยู่ (กดเพื่อปิด)'),
  ('en', 'ntpg000', 'sound.on', 'Notification sound is on (click to turn off)'),
  ('th', 'ntpg000', 'sound.off', 'ปิดเสียงแจ้งเตือนอยู่ (กดเพื่อเปิด)'),
  ('en', 'ntpg000', 'sound.off', 'Notification sound is off (click to turn on)')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
