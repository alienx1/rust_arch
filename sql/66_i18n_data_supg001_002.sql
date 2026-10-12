-- Single-language name column headers of SUPG001 / SUPG002 tables (names now follow the UI language).
-- Idempotent (ON CONFLICT DO NOTHING). PostgreSQL 17 has no uuidv7(): ids from unix-ms + random bits.

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg001', 'column.name_one', 'ชื่อระบบ'),
  ('en', 'supg001', 'column.name_one', 'System name'),
  ('th', 'supg001', 'module.column.name_one', 'ชื่อโมดูล'),
  ('en', 'supg001', 'module.column.name_one', 'Module name'),
  ('th', 'supg002', 'column.name_one', 'ชื่อเมนู'),
  ('en', 'supg002', 'column.name_one', 'Menu name')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
