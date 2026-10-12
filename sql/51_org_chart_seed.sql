-- Starting org chart: one organization -> company -> branch -> department.
--
-- ARCH / ARCH-HQ / HQ / IT are a starting point to build on (named like placeholders on purpose):
-- replace them with the real structure from the DBPG000-DBPG002 screens. The permission groups
-- and accounts of 53_/54_ hang on these rows, so keep them or re-point those seeds.
--
-- First file that creates tenant rows; needs the reference data before it (02-50: languages,
-- master lookups). Ids are hand-assigned in a `019ca1ec-0000-7000-8NNN-...` block so they stay
-- stable and greppable across re-seeds, one NNN group per table. Idempotent: `ON CONFLICT DO
-- NOTHING` with no target on purpose -- these tables carry a natural-key unique index as well as
-- the primary key, and a row already present under either is fine to leave alone.

------------------------------------------------------
-- Organization -> Company -> Branch -> Department
------------------------------------------------------
-- `ON CONFLICT DO NOTHING` with no target deliberately: these tables carry a
-- natural-key unique index as well as the primary key, and a row already
-- present under either is equally fine to leave alone.
INSERT INTO "db_organization" ("id", "code", "registration_number", "active") VALUES
('019ca1ec-0000-7000-8001-000000000001', 'ARCH', '0000000000000', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_organization_label" ("organization_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8001-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'องค์กร ARCH'),
('019ca1ec-0000-7000-8001-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'ARCH Organization')
ON CONFLICT DO NOTHING;

INSERT INTO "db_company" ("id", "organization_id", "code", "master_lookup_id", "active") VALUES
('019ca1ec-0000-7000-8002-000000000001', '019ca1ec-0000-7000-8001-000000000001', 'ARCH-HQ', '019ca1eb-54c0-7000-8007-000000000002', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_company_label" ("company_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8002-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'บริษัท ARCH จำกัด'),
('019ca1ec-0000-7000-8002-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'ARCH Company Limited')
ON CONFLICT DO NOTHING;

-- db_branch.address_group_id / .contact_group_id are NOT NULL, so both groups
-- have to exist before the branch. They stay empty: a head office with no
-- recorded address is a normal incomplete master-data record.
INSERT INTO "db_address_group" ("id", "active") VALUES
('019ca1ec-0000-7000-8003-000000000001', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_contact_group" ("id", "active") VALUES
('019ca1ec-0000-7000-8004-000000000001', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_branch" ("id", "organization_id", "company_id", "code", "master_lookup_id", "address_group_id", "contact_group_id", "active") VALUES
('019ca1ec-0000-7000-8005-000000000001', '019ca1ec-0000-7000-8001-000000000001', '019ca1ec-0000-7000-8002-000000000001', 'HQ', '019ca1eb-54c0-7000-8008-000000000001', '019ca1ec-0000-7000-8003-000000000001', '019ca1ec-0000-7000-8004-000000000001', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_branch_label" ("branch_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8005-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'สำนักงานใหญ่'),
('019ca1ec-0000-7000-8005-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Head Office')
ON CONFLICT DO NOTHING;

INSERT INTO "db_department" ("id", "organization_id", "company_id", "branch_id", "code", "active") VALUES
('019ca1ec-0000-7000-8006-000000000001', '019ca1ec-0000-7000-8001-000000000001', '019ca1ec-0000-7000-8002-000000000001', '019ca1ec-0000-7000-8005-000000000001', 'IT', true)
ON CONFLICT DO NOTHING;

INSERT INTO "db_department_label" ("department_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8006-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ฝ่ายเทคโนโลยีสารสนเทศ'),
('019ca1ec-0000-7000-8006-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Information Technology')
ON CONFLICT DO NOTHING;
