-- Contacts of organizations and companies, and the contact types for tax id, fax and ID card.
-- Idempotent; sql/01_ysql.sql carries the same columns for a fresh database.
-- (The tax id used to be typed into db_organization.registration_number; it is a contact now.)

ALTER TABLE "db_organization" ADD COLUMN IF NOT EXISTS "contact_group_id" uuid;
ALTER TABLE "db_company" ADD COLUMN IF NOT EXISTS "contact_group_id" uuid;
COMMENT ON COLUMN "db_organization"."registration_number" IS 'Registration number of the organization with this system (not the tax id: that is a contact of type TAX_ID)';
COMMENT ON COLUMN "db_organization"."contact_group_id" IS 'Organization contacts (phone, fax, email, tax id ...)';
COMMENT ON COLUMN "db_company"."contact_group_id" IS 'Company contacts (phone, fax, email, tax id ...)';

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'db_organization_contact_group_id_fkey') THEN
    ALTER TABLE "db_organization" ADD CONSTRAINT "db_organization_contact_group_id_fkey"
      FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'db_company_contact_group_id_fkey') THEN
    ALTER TABLE "db_company" ADD CONSTRAINT "db_company_contact_group_id_fkey"
      FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;
  END IF;
END $$;

-- Contact types (category `contact_type`; PHONE / EMAIL / LINE already exist)
WITH t(id, code, th, en) AS (VALUES
  ('019ca1eb-54c0-7000-8005-000000000004'::uuid, 'TAX_ID',  'เลขประจำตัวผู้เสียภาษี', 'Tax ID'),
  ('019ca1eb-54c0-7000-8005-000000000005'::uuid, 'FAX',     'โทรสาร',                 'Fax'),
  ('019ca1eb-54c0-7000-8005-000000000006'::uuid, 'ID_CARD', 'เลขบัตรประชาชน',         'ID card number')
), ins AS (
  INSERT INTO "db_master_lookup" ("id", "category", "code", "active")
  SELECT t.id, 'contact_type', t.code, true FROM t
  WHERE NOT EXISTS (SELECT 1 FROM "db_master_lookup" m WHERE m."category" = 'contact_type' AND m."code" = t.code)
  RETURNING id
)
SELECT count(*) FROM ins;

INSERT INTO "db_master_lookup_label" ("master_lookup_id", "language_id", "label")
SELECT m."id", l."id", CASE l."code" WHEN 'th' THEN t.th ELSE t.en END
FROM (VALUES ('TAX_ID', 'เลขประจำตัวผู้เสียภาษี', 'Tax ID'), ('FAX', 'โทรสาร', 'Fax'), ('ID_CARD', 'เลขบัตรประชาชน', 'ID card number')) AS t(code, th, en)
JOIN "db_master_lookup" m ON m."category" = 'contact_type' AND m."code" = t.code
JOIN "db_language" l ON l."code" IN ('th', 'en')
WHERE NOT EXISTS (SELECT 1 FROM "db_master_lookup_label" x WHERE x."master_lookup_id" = m."id" AND x."language_id" = l."id");
