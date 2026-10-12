-- Single "name" column headers for DBPG004 / DBPG005 tables (data names follow the UI language).
-- Idempotent (ON CONFLICT DO NOTHING).

WITH t(lang, cat, key, value) AS (VALUES
  ('th', 'dbpg004', 'column.name', 'ชื่อ'),
  ('en', 'dbpg004', 'column.name', 'Name'),
  ('th', 'dbpg005', 'column.name_single', 'ชื่อ'),
  ('en', 'dbpg005', 'column.name_single', 'Name')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.cat, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
