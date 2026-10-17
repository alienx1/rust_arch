-- SUPG004: edit an assignment (group @ scope) in place. Idempotent; ids are v7-style
-- (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg004', 'form.assignments_add', '+ เพิ่มกลุ่มสิทธิ์และขอบเขต'),
  ('en', 'supg004', 'form.assignments_add', '+ Add a permission group and scope'),
  ('th', 'supg004', 'form.assignments_empty_add', '+ เพิ่มกลุ่มสิทธิ์และขอบเขต (ยังไม่มีรายการ)'),
  ('en', 'supg004', 'form.assignments_empty_add', '+ Add a permission group and scope (none yet)'),
  ('th', 'supg004', 'form.edit', 'แก้ไข'),
  ('en', 'supg004', 'form.edit', 'Edit'),
  ('th', 'supg004', 'form.cancel_edit', 'ยกเลิก'),
  ('en', 'supg004', 'form.cancel_edit', 'Cancel'),
  ('th', 'supg004', 'form.save_assignment', 'บันทึกการแก้ไข ({level})'),
  ('en', 'supg004', 'form.save_assignment', 'Save changes ({level})')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
