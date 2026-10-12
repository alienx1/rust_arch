-- UI texts the web app's ApiClient composes around server errors (category `common`,
-- keys `error.*`). Placeholders: {what} = the data's name, {error} = the server's message,
-- {status} = HTTP status. Idempotent (ON CONFLICT DO NOTHING).
-- PostgreSQL 17 has no uuidv7(): ids are built from a unix-ms timestamp + random bits.

WITH t(lang, key, value) AS (VALUES
  ('th', 'error.connect', 'ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้'),
  ('en', 'error.connect', 'Cannot reach the server'),
  ('th', 'error.parse', 'แปลงข้อมูล{what}ล้มเหลว: {error}'),
  ('en', 'error.parse', 'Could not read the {what} data: {error}'),
  ('th', 'error.fetch_failed', 'ไม่สามารถดึงข้อมูล{what}ได้ (HTTP {status})'),
  ('en', 'error.fetch_failed', 'Could not load {what} (HTTP {status})'),
  ('th', 'error.parse_create', 'แปลงข้อมูลการสร้าง{what}ล้มเหลว: {error}'),
  ('en', 'error.parse_create', 'Could not read the result of creating {what}: {error}'),
  ('th', 'error.parse_update', 'แปลงข้อมูลการแก้ไข{what}ล้มเหลว: {error}'),
  ('en', 'error.parse_update', 'Could not read the result of updating {what}: {error}'),
  ('th', 'error.create_failed', 'สร้าง{what}ไม่สำเร็จ: {error}'),
  ('en', 'error.create_failed', 'Could not create {what}: {error}'),
  ('th', 'error.update_failed', 'แก้ไข{what}ไม่สำเร็จ: {error}'),
  ('en', 'error.update_failed', 'Could not update {what}: {error}'),
  ('th', 'error.delete_failed', 'ลบ{what}ไม่สำเร็จ: {error}'),
  ('en', 'error.delete_failed', 'Could not delete {what}: {error}'),
  ('th', 'error.forbidden', 'ไม่มีสิทธิ์แก้ไขข้อมูล'),
  ('en', 'error.forbidden', 'You are not allowed to change this data'),
  ('th', 'error.gone', 'ไม่พบข้อมูล (อาจถูกลบไปแล้ว)'),
  ('en', 'error.gone', 'Not found (it may already be deleted)')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", 'common', t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
