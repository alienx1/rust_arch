-- SUPG005: account status filter (active / suspended) next to the session filter.
-- Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'supg005', 'filter.account_status', 'สถานะบัญชี:'),
  ('en', 'supg005', 'filter.account_status', 'Account status:'),
  ('th', 'supg005', 'filter.account_all', 'ทั้งหมด'),
  ('en', 'supg005', 'filter.account_all', 'All'),
  ('th', 'supg005', 'filter.account_active', 'ใช้งานได้'),
  ('en', 'supg005', 'filter.account_active', 'Active'),
  ('th', 'supg005', 'filter.account_suspended', 'ถูกระงับ'),
  ('en', 'supg005', 'filter.account_suspended', 'Suspended')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
