-- UI texts of the SUPG003 permission-matrix filters (module, menu group, search).
-- Idempotent (ON CONFLICT DO NOTHING); PostgreSQL 17 has no uuidv7(), ids are built from a
-- unix-ms timestamp + random bits.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg003', 'matrix.filter_module', 'โมดูล:'),
  ('en', 'supg003', 'matrix.filter_module', 'Module:'),
  ('th', 'supg003', 'matrix.filter_group', 'กลุ่มเมนู:'),
  ('en', 'supg003', 'matrix.filter_group', 'Menu group:'),
  ('th', 'supg003', 'matrix.filter_all', 'ทั้งหมด'),
  ('en', 'supg003', 'matrix.filter_all', 'All'),
  ('th', 'supg003', 'matrix.search_placeholder', 'ค้นหารหัสหรือชื่อโปรแกรม...'),
  ('en', 'supg003', 'matrix.search_placeholder', 'Search program code or name...'),
  ('th', 'supg003', 'matrix.empty_filtered', 'ไม่พบโปรแกรมที่ตรงกับตัวกรอง'),
  ('en', 'supg003', 'matrix.empty_filtered', 'No programs match the filters')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
