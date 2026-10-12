-- ALPG000: the program filter is a dropdown fed by `GET /al/alpg000/programs`.
-- Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'alpg000', 'filter.program_all', 'ทั้งหมด'),
  ('en', 'alpg000', 'filter.program_all', 'All'),
  ('th', 'alpg000', 'what.programs', 'รายการโปรแกรม'),
  ('en', 'alpg000', 'what.programs', 'the program list')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
