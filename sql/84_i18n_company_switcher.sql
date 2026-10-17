-- Company and branch switcher in the sidebar brand. Idempotent; ids are v7-style (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'common', 'company.all', 'ทุกบริษัท (ภาพรวม)'),
  ('en', 'common', 'company.all', 'All companies (overview)'),
  ('th', 'common', 'company.switcher', 'บริษัทที่กำลังดู'),
  ('en', 'common', 'company.switcher', 'Company being viewed'),
  ('th', 'common', 'branch.all', 'ทุกสาขา'),
  ('en', 'common', 'branch.all', 'All branches'),
  ('th', 'common', 'branch.switcher', 'สาขาที่กำลังดู'),
  ('en', 'common', 'branch.switcher', 'Branch being viewed'),
  ('th', 'common', 'company.own_branch_only', 'แสดงข้อมูลเฉพาะสาขาของคุณ'),
  ('en', 'common', 'company.own_branch_only', 'Showing your own branch only')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
