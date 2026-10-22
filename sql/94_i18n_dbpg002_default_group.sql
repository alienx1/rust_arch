-- DBPG002: a department's default permission group (form texts). Idempotent.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'dbpg002', 'dept.form.default_group_label', 'กลุ่มสิทธิ์เริ่มต้นของแผนก (Default Permission Group)'),
  ('en', 'dbpg002', 'dept.form.default_group_label', 'Default permission group of the department'),
  ('th', 'dbpg002', 'dept.form.default_group_none', '— ไม่กำหนด —'),
  ('en', 'dbpg002', 'dept.form.default_group_none', '— none —'),
  ('th', 'dbpg002', 'dept.form.default_group_hint', 'บัญชีของพนักงานในแผนกนี้จะได้กลุ่มสิทธิ์นี้ภายในขอบเขตสาขาของแผนกโดยอัตโนมัติ ทั้งบัญชีที่สร้างใหม่และบัญชีที่มีอยู่ (เพิ่มให้ ไม่ลบสิทธิ์เดิม และแก้ไขได้ที่หน้าบัญชีผู้ใช้) กลุ่ม ADMIN ใช้เป็นค่าเริ่มต้นไม่ได้'),
  ('en', 'dbpg002', 'dept.form.default_group_hint', 'The accounts of this department''s employees get this group at the department''s branch automatically, both new accounts and existing ones (added, nothing is taken away, and it can be edited on the user accounts page). The ADMIN group cannot be the default.'),
  ('th', 'dbpg002', 'what.permission_group', 'กลุ่มสิทธิ์'),
  ('en', 'dbpg002', 'what.permission_group', 'permission groups')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
