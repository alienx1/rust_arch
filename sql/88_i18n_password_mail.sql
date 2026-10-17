-- SUPG004: password mail / forced password change labels. Idempotent.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg004', 'action.password_mail', 'ส่งอีเมลตั้งรหัสผ่าน'),
  ('en', 'supg004', 'action.password_mail', 'Send password e-mail'),
  ('th', 'supg004', 'action.force_change', 'บังคับเปลี่ยนรหัสผ่าน'),
  ('en', 'supg004', 'action.force_change', 'Force password change'),
  ('th', 'supg004', 'status.must_change', 'รอตั้งรหัสผ่าน'),
  ('en', 'supg004', 'status.must_change', 'Password pending'),
  ('th', 'supg004', 'confirm.force_change', 'บังคับ {username} เปลี่ยนรหัสผ่าน? ระบบจะออกจากระบบทุกอุปกรณ์ และเข้าใช้งานไม่ได้จนกว่าจะตั้งรหัสผ่านใหม่'),
  ('en', 'supg004', 'confirm.force_change', 'Force {username} to change the password? They are signed out everywhere and cannot sign in until a new password is set.'),
  ('th', 'supg004', 'mail.sent', 'ส่งอีเมลตั้งรหัสผ่านของ {username} เข้าคิวส่งไปที่ {to} แล้ว (ถ้าส่งไม่สำเร็จ ดูที่ log ของ api-notification)'),
  ('en', 'supg004', 'mail.sent', 'Password e-mail for {username} queued for {to} (if delivery fails, see the api-notification log)'),
  ('th', 'supg004', 'mail.failed', 'ส่งอีเมลของ {username} ไม่สำเร็จ ({error}) — ส่งลิงก์ด้านล่างให้ผู้ใช้ด้วยช่องทางอื่น'),
  ('en', 'supg004', 'mail.failed', 'The e-mail for {username} could not be sent ({error}) — pass the link below to the user another way'),
  ('th', 'supg004', 'form.password_section', 'รหัสผ่าน'),
  ('en', 'supg004', 'form.password_section', 'Password'),
  ('th', 'supg004', 'form.password_hint', 'ส่งอีเมล: ผู้ใช้ตั้งรหัสผ่านใหม่ได้ รหัสเดิมยังใช้ได้จนกว่าจะตั้งใหม่ • บังคับเปลี่ยน: ออกจากระบบทุกอุปกรณ์และเข้าไม่ได้จนกว่าจะตั้งรหัสผ่านใหม่'),
  ('en', 'supg004', 'form.password_hint', 'Send e-mail: the user can choose a new password, the current one works until then • Force change: signed out everywhere and cannot sign in until a new password is set'),
  ('th', 'supg004', 'mail.link_hint', 'ลิงก์ใช้ได้ครั้งเดียวและจะไม่แสดงซ้ำ'),
  ('en', 'supg004', 'mail.link_hint', 'The link works once and is not shown again')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
