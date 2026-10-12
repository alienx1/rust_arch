-- SUPG004 assignments editor (group @ scope unit) labels, th + en.
-- Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg004', 'scope.organization', 'องค์กร (Organization)'),
  ('en', 'supg004', 'scope.organization', 'Organization'),
  ('th', 'supg004', 'scope.company', 'บริษัท (Company)'),
  ('en', 'supg004', 'scope.company', 'Company'),
  ('th', 'supg004', 'scope.branch', 'สาขา (Branch)'),
  ('en', 'supg004', 'scope.branch', 'Branch'),
  ('th', 'supg004', 'scope.department', 'แผนก (Department)'),
  ('en', 'supg004', 'scope.department', 'Department'),
  ('th', 'supg004', 'what.scope', 'ขอบเขตข้อมูล'),
  ('en', 'supg004', 'what.scope', 'Scope units'),
  ('th', 'supg004', 'form.err_no_options', 'ยังไม่มีข้อมูลในระดับนี้ให้เลือก'),
  ('en', 'supg004', 'form.err_no_options', 'There is nothing to pick at this level yet'),
  ('th', 'supg004', 'form.err_assignment_duplicate', 'เพิ่มกลุ่มสิทธิ์นี้ในขอบเขตนี้ไปแล้ว'),
  ('en', 'supg004', 'form.err_assignment_duplicate', 'This group is already assigned at this scope'),
  ('th', 'supg004', 'form.err_assignment_required', 'กรุณาเพิ่มกลุ่มสิทธิ์และขอบเขตอย่างน้อย 1 รายการ'),
  ('en', 'supg004', 'form.err_assignment_required', 'Add at least one group and scope'),
  ('th', 'supg004', 'form.err_group', 'กรุณาเลือกกลุ่มสิทธิ์'),
  ('en', 'supg004', 'form.err_group', 'Please pick a permission group'),
  ('th', 'supg004', 'form.assignments_heading', 'กลุ่มสิทธิ์และขอบเขต ({count} รายการ) *'),
  ('en', 'supg004', 'form.assignments_heading', 'Groups and scopes ({count}) *'),
  ('th', 'supg004', 'form.assignments_empty', 'ยังไม่มีรายการ — เลือกกลุ่มสิทธิ์และขอบเขตด้านล่างแล้วกด “เพิ่ม” (เพิ่มได้หลายรายการ)'),
  ('en', 'supg004', 'form.assignments_empty', 'Nothing yet: pick a group and a scope below and press "Add" (several allowed)'),
  ('th', 'supg004', 'form.primary', '(หลัก)'),
  ('en', 'supg004', 'form.primary', '(primary)'),
  ('th', 'supg004', 'form.remove', 'ลบ'),
  ('en', 'supg004', 'form.remove', 'Remove'),
  ('th', 'supg004', 'form.group', 'กลุ่มสิทธิ์ (Permission group) *'),
  ('en', 'supg004', 'form.group', 'Permission group *'),
  ('th', 'supg004', 'form.scope_level', 'ระดับขอบเขต (Scope level)'),
  ('en', 'supg004', 'form.scope_level', 'Scope level'),
  ('th', 'supg004', 'form.add_assignment', '+ เพิ่ม ({level})'),
  ('en', 'supg004', 'form.add_assignment', '+ Add ({level})'),
  ('th', 'supg004', 'form.admin_group_notice', '🔒 กลุ่มผู้ดูแลระบบ: ต้องมีสิทธิ์ APPROVE ในโปรแกรมนี้จึงจะกำหนดได้'),
  ('en', 'supg004', 'form.admin_group_notice', 'Admin group: assigning it needs APPROVE on this program'),
  ('th', 'supg004', 'column.permission_group', 'กลุ่มสิทธิ์ / ขอบเขต'),
  ('en', 'supg004', 'column.permission_group', 'Group / scope')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
