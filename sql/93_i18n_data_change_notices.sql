-- NTPG000: notices that tell a user how their create / update / delete went. Idempotent.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'ntpg000', 'kind.data_created.title', 'บันทึกข้อมูลใหม่สำเร็จ'),
  ('en', 'ntpg000', 'kind.data_created.title', 'Saved successfully'),
  ('th', 'ntpg000', 'kind.data_created.body', 'สร้าง {target} ในโปรแกรม {program} เรียบร้อย'),
  ('en', 'ntpg000', 'kind.data_created.body', 'Created {target} in {program}'),
  ('th', 'ntpg000', 'kind.data_updated.title', 'แก้ไขข้อมูลสำเร็จ'),
  ('en', 'ntpg000', 'kind.data_updated.title', 'Changes saved'),
  ('th', 'ntpg000', 'kind.data_updated.body', 'แก้ไข {target} ในโปรแกรม {program} เรียบร้อย'),
  ('en', 'ntpg000', 'kind.data_updated.body', 'Updated {target} in {program}'),
  ('th', 'ntpg000', 'kind.data_deleted.title', 'ลบข้อมูลสำเร็จ'),
  ('en', 'ntpg000', 'kind.data_deleted.title', 'Deleted successfully'),
  ('th', 'ntpg000', 'kind.data_deleted.body', 'ลบ {target} ในโปรแกรม {program} เรียบร้อย'),
  ('en', 'ntpg000', 'kind.data_deleted.body', 'Deleted {target} in {program}'),
  ('th', 'ntpg000', 'kind.data_failed.title', 'บันทึกข้อมูลไม่สำเร็จ'),
  ('en', 'ntpg000', 'kind.data_failed.title', 'Could not save'),
  ('th', 'ntpg000', 'kind.data_failed.body', 'ทำรายการ {target} ในโปรแกรม {program} ไม่สำเร็จ (รหัส {status})'),
  ('en', 'ntpg000', 'kind.data_failed.body', 'The change to {target} in {program} failed (code {status})')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
