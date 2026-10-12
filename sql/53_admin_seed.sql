-- The administrator: permission group ADMIN (READ / WRITE / APPROVE on every active program),
-- its employee record, the su_user account `admin` and its assignment (ADMIN at the whole ARCH
-- organization).
--
-- Needs 51_org_chart_seed.sql (org chart) and 52_program_catalogue.sql (programs), and runs
-- before anything that depends on an admin existing. Re-runnable at any time on a loaded
-- database to recreate the account after it was deleted or deactivated, re-grant every
-- permission after an experiment revoked some, or pick up programs added to the catalogue since
-- the last run.
--
--   make seed                                  # runs it with the rest of sql/
--   psql ... -v admin_password_hash='$argon2id$...' -f sql/53_admin_seed.sql
--
-- Two psql variables override the defaults -- `admin_username` and `admin_password_hash` -- each
-- falling back below if unset. Login password (Kratos): Admin@2026 -- CHANGE IT before any
-- deployment that is not local development.
--
-- Rows use the hand-assigned `019ca1ec-0000-7000-8NNN-...` ids of 51_, one NNN group per table.
-- `su_user` is the one row with two identities that both matter: the pinned id and the unique
-- username. Section 4 matches on either, so renaming the bootstrap account works.

\if :{?admin_username}
\else
\set admin_username 'admin'
\endif

-- Argon2id (19 MiB, t=2, p=1 -- OWASP's recommended parameters) for the
-- plaintext "Admin@2026", as a PHC string. This hash is public in this repository: CHANGE IT BEFORE ANY
-- DEPLOYMENT THAT IS NOT LOCAL DEVELOPMENT. Generate a replacement through the
-- application's own hasher, so the seed can never drift from the format the
-- code writes and the tests pin:
--   cargo run -p infrastructure --example hash_password -- '<new password>'
\if :{?admin_password_hash}
\else
\set admin_password_hash '$argon2id$v=19$m=19456,t=2,p=1$7yHKoGvum/+MMiI40op3dA$CtIeWHzXWG53d9uZwvnRqHhiGuehhGO1sQqWqgUz2cA'
\endif

------------------------------------------------------
-- 1. Administrator permission group
------------------------------------------------------
-- A group is only a set of permissions; where ADMIN applies (the whole ARCH organization) is the
-- account's assignment further down.
INSERT INTO "su_permission_group" ("id", "code", "active") VALUES
('019ca1ec-0000-7000-800a-000000000001', 'ADMIN', true)
ON CONFLICT ("id") DO UPDATE SET "code" = EXCLUDED."code", "active" = true;

------------------------------------------------------
-- 2. READ / WRITE / APPROVE on every active program
------------------------------------------------------
-- Derived from db_program rather than written out row by row, so it stays correct as the
-- catalogue grows: a program added after this file was written is granted the next time it runs.
-- Folders are included -- the menu query restores a permitted child's ancestors on its own, but
-- granting the folder keeps su_permission a complete statement of what the group may reach.
-- Where they apply is the user's assignment (su_user_assignment), not the group.
--
-- THE DISTINCT ON IS LOAD-BEARING. 05_db_master_lookup_data.sql repeats its INSERT blocks, so
-- db_master_lookup holds several rows per permission_type code, each with its own id. Cross-joining
-- all of them would write several permission rows per program per action. Picking the lowest id
-- per code gives one row per grant. Application code sidesteps the duplicates entirely by
-- resolving actions on `code`, never on a lookup id.
INSERT INTO "su_permission" ("id", "permission_group_id", "system_id", "module_id", "program_id", "master_lookup_id", "active")
SELECT encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
       '019ca1ec-0000-7000-800a-000000000001',
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
           AND upper("code") IN ('READ', 'WRITE', 'APPROVE')
         ORDER BY upper("code"), "id"
       ) a
 WHERE p."active"
ON CONFLICT ("permission_group_id", "system_id", "module_id", "program_id", "master_lookup_id")
DO UPDATE SET "active" = true;

------------------------------------------------------
-- 3. The administrator's employee record
------------------------------------------------------
-- master_lookup_id points at 'prefix' code '002' (นาย / Mr.) in the full
-- 05_db_master_lookup_data.sql prefix catalogue (there is no separate small prefix set).
INSERT INTO "db_person_name" ("id", "master_lookup_id", "active") VALUES
('019ca1ec-0000-7000-800c-000000000001', '019db80a-4ede-7138-8448-3361f4db83da', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_person_name_label" ("person_name_id", "language_id", "first_name", "middle_name", "last_name", "nickname") VALUES
('019ca1ec-0000-7000-800c-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ผู้ดูแล', NULL, 'ระบบ', NULL),
('019ca1ec-0000-7000-800c-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'System', NULL, 'Administrator', NULL)
ON CONFLICT DO NOTHING;

INSERT INTO "db_contact_group" ("id", "active") VALUES
('019ca1ec-0000-7000-800d-000000000001', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_contact" ("id", "contact_group_id", "master_lookup_id", "value", "default_contact") VALUES
('019ca1ec-0000-7000-800e-000000000001', '019ca1ec-0000-7000-800d-000000000001', '019ca1eb-54c0-7000-8005-000000000002', 'admin@arch.local', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_employee" ("id", "organization_id", "company_id", "branch_id", "department_id", "person_name_id", "address_group_id", "contact_group_id", "code", "active") VALUES
('019ca1ec-0000-7000-800f-000000000001', '019ca1ec-0000-7000-8001-000000000001', '019ca1ec-0000-7000-8002-000000000001', '019ca1ec-0000-7000-8005-000000000001', '019ca1ec-0000-7000-8006-000000000001', '019ca1ec-0000-7000-800c-000000000001', NULL, '019ca1ec-0000-7000-800d-000000000001', 'EMP0001', true)
ON CONFLICT DO NOTHING;

-- Reactivate an employee that exists but was switched off: db_employee is what
-- scopes the account under ABAC, and an inactive one would leave the admin
-- unable to see anything.
UPDATE "db_employee"
   SET "active" = true, "row_version" = "row_version" + 1
 WHERE "id" = '019ca1ec-0000-7000-800f-000000000001'
   AND NOT "active";

------------------------------------------------------
-- 4. The account
------------------------------------------------------
-- su_user is the application's side of an account (employee, permission group, active flag). The
-- login credential itself lives in Ory Kratos: `make bootstrap` (or `make seed-admin`) creates the
-- Kratos identity admin@arch.local with the password below, matched to this row by username.
-- password_hash is kept for the account record only; it is not what login checks.
--
-- Two identities have to be honoured at once: `su_user.id` is pinned and `su_user.username` is uniquely indexed. A
-- single `ON CONFLICT` can only infer one of them, and whichever it named the
-- other would raise -- renaming the account would collide on the id, and an
-- account created under a different id would collide on the username. So: update
-- whichever row already answers to either identity, preferring the pinned id,
-- then insert only if neither was there.
UPDATE "su_user" SET
    "employee_id"         = '019ca1ec-0000-7000-800f-000000000001',
    "username"            = :'admin_username',
    "permission_group_id" = '019ca1ec-0000-7000-800a-000000000001',
    "password_hash"       = :'admin_password_hash',
    "password_type"       = 'argon2id',
    "active"              = true,
    "row_version"         = "row_version" + 1
 WHERE "id" = (
        SELECT "id" FROM "su_user"
         WHERE "id" = '019ca1ec-0000-7000-8010-000000000001'
            OR "username" = :'admin_username'
         ORDER BY ("id" = '019ca1ec-0000-7000-8010-000000000001') DESC
         LIMIT 1);

INSERT INTO "su_user" ("id", "employee_id", "username", "permission_group_id", "password_hash", "password_type", "active")
SELECT '019ca1ec-0000-7000-8010-000000000001',
       '019ca1ec-0000-7000-800f-000000000001',
       :'admin_username',
       '019ca1ec-0000-7000-800a-000000000001',
       :'admin_password_hash',
       'argon2id',
       true
 WHERE NOT EXISTS (
        SELECT 1 FROM "su_user"
         WHERE "id" = '019ca1ec-0000-7000-8010-000000000001'
            OR "username" = :'admin_username');

-- The account's assignment: which group it holds WHERE (a user can hold several, SUPG004 edits
-- them). The user's primary group is the first one.
INSERT INTO "su_user_assignment" ("id", "user_id", "permission_group_id", "scope_level", "organization_id")
SELECT '019ca1ec-0000-7000-8011-000000000001', u."id", '019ca1ec-0000-7000-800a-000000000001', 'organization', '019ca1ec-0000-7000-8001-000000000001'
  FROM "su_user" u
 WHERE u."username" = :'admin_username'
ON CONFLICT DO NOTHING;

------------------------------------------------------
-- 5. What the run produced
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
 WHERE u."username" = :'admin_username'
 GROUP BY u."username", u."id", g."code", a."scope_level";
