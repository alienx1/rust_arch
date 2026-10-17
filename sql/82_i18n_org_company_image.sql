-- Image field shared by the record forms (organization / company logo, employee photo). Idempotent; ids are v7-style
-- (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'common', 'what.image', 'รูปภาพ'),
  ('en', 'common', 'what.image', 'image'),
  ('th', 'common', 'image.label', 'รูปภาพ'),
  ('en', 'common', 'image.label', 'Image'),
  ('th', 'common', 'image.save_first', 'บันทึกข้อมูลก่อน แล้วจึงเพิ่มรูปภาพได้'),
  ('en', 'common', 'image.save_first', 'Save the record first, then add an image'),
  ('th', 'common', 'image.choose', 'เลือกรูป'),
  ('en', 'common', 'image.choose', 'Choose image'),
  ('th', 'common', 'image.change', 'เปลี่ยนรูป'),
  ('en', 'common', 'image.change', 'Change image'),
  ('th', 'common', 'image.remove', 'ลบรูป'),
  ('en', 'common', 'image.remove', 'Remove image'),
  ('th', 'common', 'image.hint', 'PNG, JPEG, GIF หรือ WebP ขนาดไม่เกิน 2 MB • รูปจะถูกบันทึกทันที'),
  ('en', 'common', 'image.hint', 'PNG, JPEG, GIF or WebP up to 2 MB • the image is saved right away'),
  ('th', 'common', 'image.error.type', 'ไฟล์ต้องเป็นรูปภาพ (PNG, JPEG, GIF หรือ WebP)'),
  ('en', 'common', 'image.error.type', 'The file must be an image (PNG, JPEG, GIF or WebP)'),
  ('th', 'common', 'image.error.size', 'รูปภาพต้องไม่เกิน 2 MB'),
  ('en', 'common', 'image.error.size', 'The image must not exceed 2 MB'),
  ('th', 'common', 'image.error.read', 'อ่านไฟล์รูปภาพไม่สำเร็จ'),
  ('en', 'common', 'image.error.read', 'Could not read the image file')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
