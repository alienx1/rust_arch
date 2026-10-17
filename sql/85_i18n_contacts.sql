-- Contacts editor of the record forms (phone, fax, e-mail, Line, tax id, ID card) and the corrected
-- wording of the organization's registration number. Idempotent (updates the wording of existing
-- keys); ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'dbpg001', 'org.field.tax_id', 'เลขทะเบียนองค์กรในระบบ'),
  ('en', 'dbpg001', 'org.field.tax_id', 'Organization registration no.'),
  ('th', 'dbpg001', 'org.field.tax_id_placeholder', 'เลขที่ใช้ลงทะเบียนองค์กรกับระบบ เช่น 0000000001'),
  ('en', 'dbpg001', 'org.field.tax_id_placeholder', 'The number the organization is registered with in this system, e.g. 0000000001'),
  ('th', 'common', 'contact.title', 'ช่องทางติดต่อและเลขประจำตัว'),
  ('en', 'common', 'contact.title', 'Contacts and identification'),
  ('th', 'common', 'contact.hint_unsaved', 'บันทึกข้อมูลก่อน แล้วจึงเพิ่มช่องทางติดต่อได้'),
  ('en', 'common', 'contact.hint_unsaved', 'Save the record first, then add contacts'),
  ('th', 'common', 'contact.hint', 'โทรศัพท์ โทรสาร อีเมล ไลน์ และเลขประจำตัว บันทึกพร้อมการกดบันทึกข้อมูล'),
  ('en', 'common', 'contact.hint', 'Phone, fax, e-mail, Line and identification numbers; saved with the record'),
  ('th', 'common', 'contact.add', '+ เพิ่มช่องทางติดต่อ'),
  ('en', 'common', 'contact.add', '+ Add contact'),
  ('th', 'common', 'contact.remove', 'ลบ'),
  ('en', 'common', 'contact.remove', 'Remove'),
  ('th', 'common', 'contact.value', 'ค่า'),
  ('en', 'common', 'contact.value', 'Value'),
  ('th', 'common', 'contact.default', 'ใช้เป็นหลัก'),
  ('en', 'common', 'contact.default', 'Preferred'),
  ('th', 'common', 'contact.empty', 'ยังไม่มีช่องทางติดต่อ'),
  ('en', 'common', 'contact.empty', 'No contacts yet'),
  ('th', 'common', 'contact.type.PHONE', 'โทรศัพท์'),
  ('en', 'common', 'contact.type.PHONE', 'Phone'),
  ('th', 'common', 'contact.type.FAX', 'โทรสาร (Fax)'),
  ('en', 'common', 'contact.type.FAX', 'Fax'),
  ('th', 'common', 'contact.type.EMAIL', 'อีเมล'),
  ('en', 'common', 'contact.type.EMAIL', 'E-mail'),
  ('th', 'common', 'contact.type.LINE', 'ไลน์ไอดี'),
  ('en', 'common', 'contact.type.LINE', 'Line ID'),
  ('th', 'common', 'contact.type.TAX_ID', 'เลขประจำตัวผู้เสียภาษี (Tax ID)'),
  ('en', 'common', 'contact.type.TAX_ID', 'Tax ID'),
  ('th', 'common', 'contact.type.ID_CARD', 'เลขบัตรประชาชน'),
  ('en', 'common', 'contact.type.ID_CARD', 'ID card number'),
  ('th', 'common', 'contact.placeholder.PHONE', 'เช่น 02-123-4567'),
  ('en', 'common', 'contact.placeholder.PHONE', 'e.g. 02-123-4567'),
  ('th', 'common', 'contact.placeholder.FAX', 'เช่น 02-123-4568'),
  ('en', 'common', 'contact.placeholder.FAX', 'e.g. 02-123-4568'),
  ('th', 'common', 'contact.placeholder.EMAIL', 'name@company.com'),
  ('en', 'common', 'contact.placeholder.EMAIL', 'name@company.com'),
  ('th', 'common', 'contact.placeholder.LINE', 'ไลน์ไอดี'),
  ('en', 'common', 'contact.placeholder.LINE', 'Line ID'),
  ('th', 'common', 'contact.placeholder.TAX_ID', 'เช่น 0105550000001'),
  ('en', 'common', 'contact.placeholder.TAX_ID', 'e.g. 0105550000001'),
  ('th', 'common', 'contact.placeholder.ID_CARD', 'เช่น 1101700203451'),
  ('en', 'common', 'contact.placeholder.ID_CARD', 'e.g. 1101700203451'),
  ('th', 'common', 'contact.saved_partly', 'บันทึกข้อมูลแล้ว แต่บันทึกช่องทางติดต่อไม่สำเร็จ'),
  ('en', 'common', 'contact.saved_partly', 'The record was saved, but its contacts were not'),
  ('th', 'common', 'what.contacts', 'ช่องทางติดต่อ'),
  ('en', 'common', 'what.contacts', 'contacts')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO UPDATE SET "value" = EXCLUDED."value";
