-- A second account scoped narrower than the administrator, to exercise ABAC by hand or in tests.
--
-- The bootstrap admin (53_) is scoped at 'organization' -- the widest tier -- and holds
-- READ/WRITE/APPROVE on every program, so nothing it does exercises scope filtering or
-- write-gating. This account is the opposite on both axes at once:
--
--   * group ABAC_TEST_DEPT assigned at the IT department only, so writes and deletes on any other
--     branch / department are refused;
--   * READ only: every write / delete is refused, and the web app hides those buttons.
--
-- Needs 51_ (org chart) and 52_ (programs). Login (Kratos): test_abac / test_abac@arch.local,
-- password Test@2026, created by `make bootstrap` / `make seed-test-users`.
--   psql ... -v abac_test_password_hash='$argon2id$...' -f sql/54_abac_test_seed.sql
--
-- Rows use a dedicated `019ca1ec-0000-7000-9NNN-...` block, distinct from 53_'s `...8NNN...`
-- bootstrap block, so the two seeds can never collide on id. Idempotent.

\if :{?abac_test_username}
\else
\set abac_test_username 'test_abac'
\endif

-- Argon2id (19 MiB, t=2, p=1) for the plaintext "Test@2026". Public in this
-- repository like the admin's -- change it before anything but local dev.
--   cargo run -p infrastructure --example hash_password -- '<new password>'
\if :{?abac_test_password_hash}
\else
\set abac_test_password_hash '$argon2id$v=19$m=19456,t=2,p=1$jdAeRnB6OhqTeAA5N06ncg$d0N3/8CHkrJTfkU25huBrEXVqGwOVHGZJ1kOC43AC+8'
\endif

------------------------------------------------------
-- 1. READ-only permission group
------------------------------------------------------
-- Where it applies (the IT department of ARCH-HQ / HQ) is the account's assignment below.
INSERT INTO "su_permission_group" ("id", "code", "active") VALUES
('019ca1ec-0000-7000-900a-000000000001', 'ABAC_TEST_DEPT', true)
ON CONFLICT ("id") DO UPDATE SET "code" = EXCLUDED."code", "active" = true;

-- Same derivation as 53_ section 2, but READ only and against this group --
-- see that file's comment for why DISTINCT ON is load-bearing.
INSERT INTO "su_permission" ("id", "permission_group_id", "system_id", "module_id", "program_id", "master_lookup_id", "active")
SELECT encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
       '019ca1ec-0000-7000-900a-000000000001',
       p."system_id",
       p."module_id",
       p."id",
       a."id",
       true
  FROM "db_program" p
 CROSS JOIN (
        SELECT DISTINCT ON (upper("code")) "id"
          FROM "db_master_lookup"
         WHERE "category" = 'permission_type'
           AND upper("code") = 'READ'
         ORDER BY upper("code"), "id"
       ) a
 WHERE p."active"
ON CONFLICT ("permission_group_id", "system_id", "module_id", "program_id", "master_lookup_id")
DO UPDATE SET "active" = true;

-- Revoke any WRITE/APPROVE this group picked up from an earlier run (e.g. a
-- manual grant during testing) -- the group's whole point is READ-only.
UPDATE "su_permission" pm
   SET "active" = false, "row_version" = pm."row_version" + 1
  FROM "db_master_lookup" a
 WHERE pm."master_lookup_id" = a."id"
   AND pm."permission_group_id" = '019ca1ec-0000-7000-900a-000000000001'
   AND a."category" = 'permission_type'
   AND upper(a."code") IN ('WRITE', 'APPROVE')
   AND pm."active";

------------------------------------------------------
-- 2. The test employee, under the IT department
------------------------------------------------------
-- master_lookup_id points at 'prefix' code '004' (นางสาว / Ms.) in the full
-- 05_db_master_lookup_data.sql prefix catalogue -- (there is no separate small prefix set).
INSERT INTO "db_person_name" ("id", "master_lookup_id", "active") VALUES
('019ca1ec-0000-7000-900c-000000000001', '019db80a-4ede-72f0-861a-87918de94e56', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_person_name_label" ("person_name_id", "language_id", "first_name", "middle_name", "last_name", "nickname") VALUES
('019ca1ec-0000-7000-900c-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ทดสอบ', NULL, 'เอแบค', NULL),
('019ca1ec-0000-7000-900c-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Test', NULL, 'Abac', NULL)
ON CONFLICT DO NOTHING;

INSERT INTO "db_employee" ("id", "organization_id", "company_id", "branch_id", "department_id", "person_name_id", "address_group_id", "contact_group_id", "code", "active") VALUES
('019ca1ec-0000-7000-900f-000000000001', '019ca1ec-0000-7000-8001-000000000001', '019ca1ec-0000-7000-8002-000000000001', '019ca1ec-0000-7000-8005-000000000001', '019ca1ec-0000-7000-8006-000000000001', '019ca1ec-0000-7000-900c-000000000001', NULL, NULL, 'EMP_ABAC_TEST', true)
ON CONFLICT DO NOTHING;

UPDATE "db_employee"
   SET "active" = true, "row_version" = "row_version" + 1
 WHERE "id" = '019ca1ec-0000-7000-900f-000000000001'
   AND NOT "active";

------------------------------------------------------
-- 3. The account
------------------------------------------------------
-- Same upsert-by-either-identity shape as 53_ section 4 -- see its comment.
UPDATE "su_user" SET
    "employee_id"         = '019ca1ec-0000-7000-900f-000000000001',
    "username"            = :'abac_test_username',
    "permission_group_id" = '019ca1ec-0000-7000-900a-000000000001',
    "password_hash"       = :'abac_test_password_hash',
    "password_type"       = 'argon2id',
    "active"              = true,
    "row_version"         = "row_version" + 1
 WHERE "id" = (
        SELECT "id" FROM "su_user"
         WHERE "id" = '019ca1ec-0000-7000-9010-000000000001'
            OR "username" = :'abac_test_username'
         ORDER BY ("id" = '019ca1ec-0000-7000-9010-000000000001') DESC
         LIMIT 1);

INSERT INTO "su_user" ("id", "employee_id", "username", "permission_group_id", "password_hash", "password_type", "active")
SELECT '019ca1ec-0000-7000-9010-000000000001',
       '019ca1ec-0000-7000-900f-000000000001',
       :'abac_test_username',
       '019ca1ec-0000-7000-900a-000000000001',
       :'abac_test_password_hash',
       'argon2id',
       true
 WHERE NOT EXISTS (
        SELECT 1 FROM "su_user"
         WHERE "id" = '019ca1ec-0000-7000-9010-000000000001'
            OR "username" = :'abac_test_username');

-- The account's assignment: which group it holds WHERE (a user can hold several, SUPG004 edits
-- them). The user's primary group is the first one.
INSERT INTO "su_user_assignment" ("id", "user_id", "permission_group_id", "scope_level", "department_id")
SELECT '019ca1ec-0000-7000-9011-000000000001', u."id", '019ca1ec-0000-7000-900a-000000000001', 'department', '019ca1ec-0000-7000-8006-000000000001'
  FROM "su_user" u
 WHERE u."username" = :'abac_test_username'
ON CONFLICT DO NOTHING;

------------------------------------------------------
-- 4. What the run produced
------------------------------------------------------
SELECT u."username",
       u."id"::text                AS "user_id",
       g."code"                    AS "permission_group",
       a."scope_level",
       count(*) FILTER (WHERE pm."active") AS "active_permissions"
  FROM "su_user" u
  JOIN "su_permission_group" g ON g."id" = u."permission_group_id"
  LEFT JOIN "su_user_assignment" a ON a."user_id" = u."id" AND a."permission_group_id" = g."id"
  LEFT JOIN "su_permission" pm ON pm."permission_group_id" = g."id"
 WHERE u."username" = :'abac_test_username'
 GROUP BY u."username", u."id", g."code", a."scope_level";
