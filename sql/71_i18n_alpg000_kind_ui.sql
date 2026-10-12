-- ALPG000: one filter chip for everything the web app reports (page views, clicks, actions).
-- Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'alpg000', 'filter.kind_ui', 'หน้าจอ (เปิดหน้า / คลิก / ดำเนินการ)'),
  ('en', 'alpg000', 'filter.kind_ui', 'UI (page views / clicks / actions)')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
