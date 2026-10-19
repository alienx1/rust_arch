-- SUPG006: password policy tab texts. Idempotent.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg006', 'description', 'กำหนดการล็อกบัญชีเมื่อใส่รหัสผ่านผิดหลายครั้ง นโยบายรหัสผ่าน และเทมเพลตอีเมลตั้งรหัสผ่าน • รหัสโปรแกรม: SUPG006'),
  ('en', 'supg006', 'description', 'Set when an account is locked after wrong passwords, the password policy and the password e-mail templates • Program code: SUPG006'),
  ('th', 'supg006', 'tab.policy', 'นโยบายรหัสผ่าน'),
  ('en', 'supg006', 'tab.policy', 'Password policy'),
  ('th', 'supg006', 'policy.title', 'นโยบายรหัสผ่าน'),
  ('en', 'supg006', 'policy.title', 'Password policy'),
  ('th', 'supg006', 'policy.description', 'ใช้กับรหัสผ่านใหม่ที่ผู้ใช้ตั้งเอง (จากลิงก์ในอีเมลหรือหน้าเปลี่ยนรหัสผ่าน) ระบบตรวจก่อนส่งให้ Kratos และบอกผู้ใช้ว่าขาดข้อไหน รหัสผ่านเดิมที่ตั้งไว้แล้วไม่ถูกตรวจย้อนหลัง จนกว่าจะเปลี่ยนรหัสผ่านครั้งถัดไป'),
  ('en', 'supg006', 'policy.description', 'Applies to a new password the user chooses (from the e-mail link or the change-password page). It is checked before it reaches Kratos and the user is told which rule it breaks. Passwords already set are not checked again until they are changed.'),
  ('th', 'supg006', 'policy.rules', 'รหัสผ่านใหม่ต้อง'),
  ('en', 'supg006', 'policy.rules', 'A new password must'),
  ('th', 'supg006', 'field.security.password.min_length.label', 'ความยาวรหัสผ่านขั้นต่ำ (ตัวอักษร)'),
  ('en', 'supg006', 'field.security.password.min_length.label', 'Minimum length (characters)'),
  ('th', 'supg006', 'field.security.password.min_length.hint', 'รหัสผ่านใหม่ต้องยาวอย่างน้อยเท่านี้ (ต่ำสุด 8 เพราะ Kratos ไม่รับสั้นกว่านี้ สูงสุด 128)'),
  ('en', 'supg006', 'field.security.password.min_length.hint', 'A new password needs at least this many characters (8 at least, because Kratos refuses less; 128 at most)'),
  ('th', 'supg006', 'flag.require_uppercase', 'ต้องมีตัวพิมพ์ใหญ่ (A-Z)'),
  ('en', 'supg006', 'flag.require_uppercase', 'Contain an uppercase letter (A-Z)'),
  ('th', 'supg006', 'flag.require_lowercase', 'ต้องมีตัวพิมพ์เล็ก (a-z)'),
  ('en', 'supg006', 'flag.require_lowercase', 'Contain a lowercase letter (a-z)'),
  ('th', 'supg006', 'flag.require_digit', 'ต้องมีตัวเลข (0-9)'),
  ('en', 'supg006', 'flag.require_digit', 'Contain a digit (0-9)'),
  ('th', 'supg006', 'flag.require_symbol', 'ต้องมีอักขระพิเศษ เช่น ! @ #'),
  ('en', 'supg006', 'flag.require_symbol', 'Contain a symbol such as ! @ #'),
  ('th', 'supg006', 'flag.forbid_identifier', 'ห้ามมีชื่อผู้ใช้หรืออีเมลอยู่ในรหัสผ่าน'),
  ('en', 'supg006', 'flag.forbid_identifier', 'Not contain the user name or e-mail name')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
