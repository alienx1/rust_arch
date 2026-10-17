-- Two-level decision (ConfirmButton) labels and the 'mail could not be sent' notification. Idempotent.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'common', 'dialog.confirm_question', 'ยืนยันการดำเนินการนี้?'),
  ('en', 'common', 'dialog.confirm_question', 'Confirm this action?'),
  ('th', 'common', 'dialog.confirm', 'ยืนยัน'),
  ('en', 'common', 'dialog.confirm', 'Confirm'),
  ('th', 'common', 'dialog.save_question', 'บันทึกข้อมูลนี้ใช่หรือไม่?'),
  ('en', 'common', 'dialog.save_question', 'Save this data?'),
  ('th', 'common', 'dialog.save_confirm', 'ยืนยันบันทึก'),
  ('en', 'common', 'dialog.save_confirm', 'Confirm save'),
  ('th', 'common', 'image.remove_confirm', 'ลบรูปนี้ใช่หรือไม่?'),
  ('en', 'common', 'image.remove_confirm', 'Remove this picture?'),
  ('th', 'supg005', 'devices.revoke_others_confirm', 'เพิกถอนเซสชันของอุปกรณ์อื่นทั้งหมดใช่หรือไม่?'),
  ('en', 'supg005', 'devices.revoke_others_confirm', 'Revoke the sessions of all other devices?'),
  ('th', 'supg005', 'devices.revoke_confirm', 'ยืนยันเพิกถอน'),
  ('en', 'supg005', 'devices.revoke_confirm', 'Confirm revoke'),
  ('th', 'supg005', 'devices.revoke_one_confirm', 'เพิกถอนเซสชันนี้?'),
  ('en', 'supg005', 'devices.revoke_one_confirm', 'Revoke this session?'),
  ('th', 'supg005', 'table.reactivate_confirm', 'เปิดใช้งานบัญชีนี้อีกครั้ง?'),
  ('en', 'supg005', 'table.reactivate_confirm', 'Switch this account on again?'),
  ('th', 'supg005', 'table.suspend_confirm', 'ระงับบัญชีนี้และเพิกถอนเซสชันทั้งหมด?'),
  ('en', 'supg005', 'table.suspend_confirm', 'Suspend this account and revoke all its sessions?'),
  ('th', 'supg005', 'table.suspend_do', 'ยืนยันระงับ'),
  ('en', 'supg005', 'table.suspend_do', 'Confirm suspend'),
  ('th', 'ntpg000', 'bell.mark_read_confirm', 'ทำเครื่องหมายว่าอ่านทุกรายการแล้ว?'),
  ('en', 'ntpg000', 'bell.mark_read_confirm', 'Mark every notification as read?'),
  ('th', 'alpg001', 'action.reprocess_confirm', 'ประมวลผลรายการนี้ใหม่?'),
  ('en', 'alpg001', 'action.reprocess_confirm', 'Process this one again?'),
  ('th', 'supg004', 'confirm.password_mail', 'ส่งอีเมลตั้งรหัสผ่านให้ {username}?'),
  ('en', 'supg004', 'confirm.password_mail', 'Send the password e-mail to {username}?'),
  ('th', 'supg004', 'confirm.send', 'ยืนยันส่ง'),
  ('en', 'supg004', 'confirm.send', 'Confirm send'),
  ('th', 'supg004', 'confirm.force_do', 'ยืนยันบังคับ'),
  ('en', 'supg004', 'confirm.force_do', 'Confirm force'),
  ('th', 'ntpg000', 'kind.mail_failed.title', 'ส่งอีเมลไม่สำเร็จ'),
  ('en', 'ntpg000', 'kind.mail_failed.title', 'E-mail could not be sent'),
  ('th', 'ntpg000', 'kind.mail_failed.body', 'ส่งอีเมลถึง {to} ไม่สำเร็จ: {reason}'),
  ('en', 'ntpg000', 'kind.mail_failed.body', 'The e-mail to {to} could not be sent: {reason}')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
