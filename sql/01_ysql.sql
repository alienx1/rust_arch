-- Database schema (PostgreSQL 17 / pgEdge).
--
-- Started as a DBML dump (dbml.dbdiagram.io) and is now maintained BY HAND: this file is the
-- source of truth, edit it directly (no DBML regeneration). It is loaded once on a fresh
-- database by `make bootstrap` / `make seed`, before every data file.

CREATE TABLE "db_language" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code" varchar(2) UNIQUE NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_language_label" (
  "language_id" uuid NOT NULL,
  "display_language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("language_id", "display_language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_label" (
  "id" uuid PRIMARY KEY NOT NULL,
  "language_id" uuid NOT NULL,
  "category" varchar(100) NOT NULL,
  "key" text NOT NULL,
  "value" text NOT NULL,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_currency" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code_3166_1" varchar(2) NOT NULL,
  "code_4217" varchar(3) NOT NULL,
  "symbol" varchar(10),
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_currency_label" (
  "currency_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("currency_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_country" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code" varchar(3) UNIQUE NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_country_label" (
  "country_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("country_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_province" (
  "id" uuid PRIMARY KEY NOT NULL,
  "country_id" uuid NOT NULL,
  "code" varchar(10) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_province_label" (
  "province_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("province_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_district" (
  "id" uuid PRIMARY KEY NOT NULL,
  "country_id" uuid NOT NULL,
  "province_id" uuid NOT NULL,
  "code" varchar(10) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_district_label" (
  "district_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("district_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_sub_district" (
  "id" uuid PRIMARY KEY NOT NULL,
  "country_id" uuid NOT NULL,
  "province_id" uuid NOT NULL,
  "district_id" uuid NOT NULL,
  "code" varchar(10) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_sub_district_label" (
  "sub_district_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("sub_district_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_postal_code" (
  "id" uuid PRIMARY KEY NOT NULL,
  "country_id" uuid NOT NULL,
  "province_id" uuid NOT NULL,
  "district_id" uuid,
  "sub_district_id" uuid,
  "postal_code" varchar(20) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_master_lookup" (
  "id" uuid PRIMARY KEY NOT NULL,
  "category" varchar(100) NOT NULL,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_master_lookup_label" (
  "master_lookup_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("master_lookup_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_address_group" (
  "id" uuid PRIMARY KEY NOT NULL,
  "active" boolean DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_address" (
  "id" uuid PRIMARY KEY NOT NULL,
  "address_group_id" uuid NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "address_line1" text NOT NULL,
  "address_line2" text,
  "province_id" uuid NOT NULL,
  "district_id" uuid NOT NULL,
  "sub_district_id" uuid NOT NULL,
  "postal_code_id" uuid NOT NULL,
  "default_address" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_contact_group" (
  "id" uuid PRIMARY KEY NOT NULL,
  "active" boolean DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_contact" (
  "id" uuid PRIMARY KEY NOT NULL,
  "contact_group_id" uuid NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "value" text NOT NULL,
  "default_contact" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_person_name" (
  "id" uuid PRIMARY KEY NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_person_name_label" (
  "person_name_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "first_name" varchar(100) NOT NULL,
  "middle_name" varchar(100),
  "last_name" varchar(100) NOT NULL,
  "nickname" varchar(300),
  PRIMARY KEY ("person_name_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_organization" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code" varchar(100) NOT NULL,
  "registration_number" varchar(100) NOT NULL,
  "contact_group_id" uuid,
  "active" boolean NOT NULL DEFAULT false,
  "image_key" varchar(500),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_organization_label" (
  "organization_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("organization_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_company" (
  "id" uuid PRIMARY KEY NOT NULL,
  "organization_id" uuid NOT NULL,
  "code" varchar(100) NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "contact_group_id" uuid,
  "active" boolean NOT NULL DEFAULT false,
  "image_key" varchar(500),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_company_label" (
  "company_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("company_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_branch" (
  "id" uuid PRIMARY KEY NOT NULL,
  "organization_id" uuid NOT NULL,
  "company_id" uuid NOT NULL,
  "code" varchar(100) NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "address_group_id" uuid NOT NULL,
  "contact_group_id" uuid NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_branch_label" (
  "branch_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("branch_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_department" (
  "id" uuid PRIMARY KEY NOT NULL,
  "organization_id" uuid NOT NULL,
  "company_id" uuid NOT NULL,
  "branch_id" uuid NOT NULL,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_department_label" (
  "department_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("department_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_system" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_system_label" (
  "system_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("system_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_module" (
  "id" uuid PRIMARY KEY NOT NULL,
  "system_id" uuid NOT NULL,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_module_label" (
  "module_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("module_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_program" (
  "id" uuid PRIMARY KEY NOT NULL,
  "system_id" uuid NOT NULL,
  "module_id" uuid NOT NULL,
  "program_id" uuid,
  "code" varchar(100) NOT NULL,
  "path" text,
  "icon" varchar(50),
  "sort_order" integer NOT NULL DEFAULT 0,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_program_label" (
  "program_id" uuid NOT NULL,
  "language_id" uuid NOT NULL,
  "label" text NOT NULL,
  PRIMARY KEY ("program_id", "language_id"),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "su_configuration" (
  "id" uuid PRIMARY KEY NOT NULL,
  "category" varchar(100) NOT NULL,
  "key" varchar(100) UNIQUE NOT NULL,
  "value" text NOT NULL,
  "description" text,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TYPE "scope_level" AS ENUM ('organization', 'company', 'branch', 'department');

-- A named set of permissions (the SUPG003 matrix). It says WHAT a holder may do; WHERE (which
-- organization / company / branch / department) is decided per user by su_user_assignment.
CREATE TABLE "su_permission_group" (
  "id" uuid PRIMARY KEY NOT NULL,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "su_permission" (
  "id" uuid PRIMARY KEY NOT NULL,
  "permission_group_id" uuid NOT NULL,
  "system_id" uuid NOT NULL,
  "module_id" uuid NOT NULL,
  "program_id" uuid NOT NULL,
  "master_lookup_id" uuid NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "db_employee" (
  "id" uuid PRIMARY KEY NOT NULL,
  "organization_id" uuid NOT NULL,
  "company_id" uuid NOT NULL,
  "branch_id" uuid NOT NULL,
  "department_id" uuid NOT NULL,
  "person_name_id" uuid NOT NULL,
  "address_group_id" uuid,
  "contact_group_id" uuid,
  "code" varchar(100) NOT NULL,
  "active" boolean NOT NULL DEFAULT false,
  "image_key" varchar(500),
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE TABLE "su_user" (
  "id" uuid PRIMARY KEY NOT NULL,
  "employee_id" uuid NOT NULL,
  "username" varchar(100) NOT NULL,
  "permission_group_id" uuid NOT NULL,
  "password_hash" varchar(255) NOT NULL DEFAULT '',
  "password_type" varchar(50) NOT NULL DEFAULT 'argon2id',
  "created_time" timestamptz NOT NULL DEFAULT now(),
  "active" boolean NOT NULL DEFAULT false,
  "must_change_password" boolean NOT NULL DEFAULT false,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1
);

-- What a user may do WHERE: one row per (permission group, scope unit). The group says which
-- programs / actions, the row says in which organization / company / branch / department they
-- apply, so one user can hold several groups at several scopes (group A at branch 1, group B at
-- branch 2). su_user.permission_group_id stays as the user's primary group (the first row); a user
-- with no row here falls back to that group's own scope.
CREATE TABLE "su_user_assignment" (
  "id" uuid PRIMARY KEY NOT NULL,
  "user_id" uuid NOT NULL,
  "permission_group_id" uuid NOT NULL,
  "scope_level" scope_level NOT NULL,
  "organization_id" uuid,
  "company_id" uuid,
  "branch_id" uuid,
  "department_id" uuid,
  "change_by" uuid,
  "row_version" bigint NOT NULL DEFAULT 1,
  CONSTRAINT "su_user_assignment_one_target_check" CHECK (
    (scope_level = 'organization' AND organization_id IS NOT NULL AND company_id IS NULL AND branch_id IS NULL AND department_id IS NULL) OR
    (scope_level = 'company' AND company_id IS NOT NULL AND organization_id IS NULL AND branch_id IS NULL AND department_id IS NULL) OR
    (scope_level = 'branch' AND branch_id IS NOT NULL AND organization_id IS NULL AND company_id IS NULL AND department_id IS NULL) OR
    (scope_level = 'department' AND department_id IS NOT NULL AND organization_id IS NULL AND company_id IS NULL AND branch_id IS NULL)
  )
);

CREATE UNIQUE INDEX ON "db_label" ("category", "key", "language_id");

CREATE UNIQUE INDEX ON "db_currency" ("code_3166_1", "code_4217");

CREATE INDEX ON "db_currency" ("code_4217");

CREATE UNIQUE INDEX ON "db_province" ("country_id", "code");

CREATE UNIQUE INDEX ON "db_district" ("province_id", "code");

CREATE UNIQUE INDEX ON "db_sub_district" ("district_id", "code");

CREATE INDEX ON "db_postal_code" ("postal_code");

CREATE INDEX ON "db_postal_code" ("province_id");

CREATE INDEX ON "db_master_lookup" ("category", "code", "active");

CREATE INDEX ON "db_address" ("address_group_id");

CREATE INDEX ON "db_address" ("address_group_id", "master_lookup_id");

CREATE INDEX ON "db_address" ("address_group_id", "default_address");

CREATE INDEX ON "db_contact" ("contact_group_id");

CREATE INDEX ON "db_contact" ("contact_group_id", "master_lookup_id");

CREATE INDEX ON "db_contact" ("contact_group_id", "default_contact");

CREATE INDEX ON "db_person_name_label" ("first_name", "middle_name", "last_name");

CREATE INDEX ON "db_person_name_label" ("first_name", "last_name");

CREATE UNIQUE INDEX ON "db_organization" ("code", "registration_number");

CREATE INDEX ON "db_organization" ("code");

CREATE UNIQUE INDEX ON "db_company" ("organization_id", "code");

CREATE UNIQUE INDEX ON "db_branch" ("organization_id", "company_id", "code");

CREATE UNIQUE INDEX ON "db_department" ("organization_id", "company_id", "branch_id", "code");

CREATE UNIQUE INDEX ON "db_system" ("code");

CREATE UNIQUE INDEX ON "db_module" ("system_id", "code");

CREATE UNIQUE INDEX ON "db_program" ("system_id", "module_id", "code");

CREATE UNIQUE INDEX ON "su_configuration" ("category", "key");

CREATE UNIQUE INDEX ON "su_permission_group" ("code");

CREATE UNIQUE INDEX ON "su_permission" ("permission_group_id", "system_id", "module_id", "program_id", "master_lookup_id");

-- List / filter queries of DBPG002 (branches & departments), DBPG003 (employees), DBPG005 (geography):
-- the unique indexes above lead with organization_id, so a filter on company_id / branch_id alone
-- cannot use them, and PostgreSQL does not index foreign keys on its own.
CREATE INDEX "idx_db_branch_company_id" ON "db_branch" ("company_id");
CREATE INDEX "idx_db_department_branch_id" ON "db_department" ("branch_id");
CREATE INDEX "idx_db_department_company_id" ON "db_department" ("company_id");
CREATE INDEX "idx_db_employee_company_id" ON "db_employee" ("company_id");
CREATE INDEX "idx_db_employee_branch_id" ON "db_employee" ("branch_id");
CREATE INDEX "idx_db_employee_department_id" ON "db_employee" ("department_id");
CREATE INDEX "idx_db_sub_district_province_id" ON "db_sub_district" ("province_id");
CREATE INDEX "idx_db_postal_code_district_id" ON "db_postal_code" ("district_id");
CREATE INDEX "idx_db_postal_code_sub_district_id" ON "db_postal_code" ("sub_district_id");

CREATE INDEX ON "db_employee" ("person_name_id");

CREATE UNIQUE INDEX ON "db_employee" ("organization_id", "company_id", "branch_id", "department_id", "code");

CREATE UNIQUE INDEX ON "su_user" ("employee_id", "username");

CREATE UNIQUE INDEX ON "su_user" ("username");

-- One row per (user, group, unit); NULLs are distinct in a plain unique index, so COALESCE the four
-- target columns into one value.
CREATE UNIQUE INDEX ON "su_user_assignment" ("user_id", "permission_group_id", (COALESCE("organization_id", "company_id", "branch_id", "department_id")));

CREATE INDEX ON "su_user_assignment" ("user_id");

COMMENT ON COLUMN "db_language"."id" IS 'Primary key of language';

COMMENT ON COLUMN "db_language"."code" IS 'ISO 639-1 language code (e.g. en, th)';

COMMENT ON COLUMN "db_language"."active" IS 'Whether this language is enabled';

COMMENT ON COLUMN "db_language_label"."language_id" IS 'Language being translated';

COMMENT ON COLUMN "db_language_label"."display_language_id" IS 'Language used for displaying the name';

COMMENT ON COLUMN "db_language_label"."label" IS 'Language name in the display language';

COMMENT ON COLUMN "db_label"."id" IS 'Primary key of system label';

COMMENT ON COLUMN "db_label"."language_id" IS 'Language of the label';

COMMENT ON COLUMN "db_label"."category" IS 'Logical label group (menu, error, validation, system)';

COMMENT ON COLUMN "db_label"."key" IS 'Key within the category, unique per language';

COMMENT ON COLUMN "db_label"."value" IS 'Localized display text';

COMMENT ON COLUMN "db_currency"."id" IS 'Primary key of currency';

COMMENT ON COLUMN "db_currency"."code_3166_1" IS 'ISO 3166-1 alpha-2 country code';

COMMENT ON COLUMN "db_currency"."code_4217" IS 'ISO 4217 currency code';

COMMENT ON COLUMN "db_currency"."symbol" IS 'Currency symbol';

COMMENT ON COLUMN "db_currency"."active" IS 'Whether currency is enabled';

COMMENT ON COLUMN "db_currency_label"."currency_id" IS 'Referenced currency';

COMMENT ON COLUMN "db_currency_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_currency_label"."label" IS 'Localized currency name';

COMMENT ON COLUMN "db_country"."id" IS 'Primary key of country';

COMMENT ON COLUMN "db_country"."code" IS 'ISO 3166-1 alpha-3 country code';

COMMENT ON COLUMN "db_country"."active" IS 'Whether country is enabled';

COMMENT ON COLUMN "db_country_label"."country_id" IS 'Referenced country';

COMMENT ON COLUMN "db_country_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_country_label"."label" IS 'Localized country name';

COMMENT ON COLUMN "db_province"."id" IS 'Primary key of province/state';

COMMENT ON COLUMN "db_province"."country_id" IS 'Parent country';

COMMENT ON COLUMN "db_province"."code" IS 'ISO 3166-2 subdivision code';

COMMENT ON COLUMN "db_province"."active" IS 'Whether province is enabled';

COMMENT ON COLUMN "db_province_label"."province_id" IS 'Referenced province';

COMMENT ON COLUMN "db_province_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_province_label"."label" IS 'Localized province name';

COMMENT ON COLUMN "db_district"."id" IS 'Primary key of district';

COMMENT ON COLUMN "db_district"."country_id" IS 'Parent country';

COMMENT ON COLUMN "db_district"."province_id" IS 'Parent province';

COMMENT ON COLUMN "db_district"."code" IS 'District code';

COMMENT ON COLUMN "db_district"."active" IS 'Whether district is enabled';

COMMENT ON COLUMN "db_district_label"."district_id" IS 'Referenced district';

COMMENT ON COLUMN "db_district_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_district_label"."label" IS 'Localized district name';

COMMENT ON COLUMN "db_sub_district"."id" IS 'Primary key of sub-district';

COMMENT ON COLUMN "db_sub_district"."country_id" IS 'Parent country';

COMMENT ON COLUMN "db_sub_district"."province_id" IS 'Parent province';

COMMENT ON COLUMN "db_sub_district"."district_id" IS 'Parent district';

COMMENT ON COLUMN "db_sub_district"."code" IS 'Sub-district code';

COMMENT ON COLUMN "db_sub_district"."active" IS 'Whether sub-district is enabled';

COMMENT ON COLUMN "db_sub_district_label"."sub_district_id" IS 'Referenced sub-district';

COMMENT ON COLUMN "db_sub_district_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_sub_district_label"."label" IS 'Localized sub-district name';

COMMENT ON COLUMN "db_postal_code"."id" IS 'Primary key of postal code';

COMMENT ON COLUMN "db_postal_code"."country_id" IS 'Country';

COMMENT ON COLUMN "db_postal_code"."province_id" IS 'Province';

COMMENT ON COLUMN "db_postal_code"."district_id" IS 'District';

COMMENT ON COLUMN "db_postal_code"."sub_district_id" IS 'Sub-district';

COMMENT ON COLUMN "db_postal_code"."postal_code" IS 'Postal or ZIP code';

COMMENT ON COLUMN "db_postal_code"."active" IS 'Whether postal code is enabled';

COMMENT ON COLUMN "db_master_lookup"."id" IS 'Primary key of lookup value';

COMMENT ON COLUMN "db_master_lookup"."category" IS 'Lookup category (address_type, permission_type, company_type)';

COMMENT ON COLUMN "db_master_lookup"."code" IS 'Lookup code within category';

COMMENT ON COLUMN "db_master_lookup"."active" IS 'Whether lookup is enabled';

COMMENT ON COLUMN "db_master_lookup_label"."master_lookup_id" IS 'Referenced lookup';

COMMENT ON COLUMN "db_master_lookup_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_master_lookup_label"."label" IS 'Localized lookup label';

COMMENT ON COLUMN "db_address_group"."id" IS 'Logical group for addresses';

COMMENT ON COLUMN "db_address_group"."active" IS 'Whether address group is active';

COMMENT ON COLUMN "db_address"."id" IS 'Primary key of address';

COMMENT ON COLUMN "db_address"."address_group_id" IS 'Owning address group';

COMMENT ON COLUMN "db_address"."master_lookup_id" IS 'Address type';

COMMENT ON COLUMN "db_address"."language_id" IS 'Language of address text';

COMMENT ON COLUMN "db_address"."address_line1" IS 'Address line 1';

COMMENT ON COLUMN "db_address"."address_line2" IS 'Address line 2';

COMMENT ON COLUMN "db_address"."province_id" IS 'Province';

COMMENT ON COLUMN "db_address"."district_id" IS 'District';

COMMENT ON COLUMN "db_address"."sub_district_id" IS 'Sub-district';

COMMENT ON COLUMN "db_address"."postal_code_id" IS 'Postal code';

COMMENT ON COLUMN "db_address"."default_address" IS 'Is default address';

COMMENT ON COLUMN "db_contact_group"."id" IS 'Logical group for contacts';

COMMENT ON COLUMN "db_contact_group"."active" IS 'Whether contact group is active';

COMMENT ON COLUMN "db_contact"."id" IS 'Primary key of contact';

COMMENT ON COLUMN "db_contact"."contact_group_id" IS 'Owning contact group';

COMMENT ON COLUMN "db_contact"."master_lookup_id" IS 'Contact type';

COMMENT ON COLUMN "db_contact"."value" IS 'Contact value (phone, email, etc.)';

COMMENT ON COLUMN "db_contact"."default_contact" IS 'Is default contact';

COMMENT ON COLUMN "db_person_name"."id" IS 'Primary key of person name';

COMMENT ON COLUMN "db_person_name"."master_lookup_id" IS 'prefix/salutation';

COMMENT ON COLUMN "db_person_name"."active" IS 'Whether person name is active';

COMMENT ON COLUMN "db_person_name_label"."person_name_id" IS 'Referenced person name';

COMMENT ON COLUMN "db_person_name_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_person_name_label"."first_name" IS 'First name';

COMMENT ON COLUMN "db_person_name_label"."middle_name" IS 'Middle name';

COMMENT ON COLUMN "db_person_name_label"."last_name" IS 'Last name';

COMMENT ON COLUMN "db_person_name_label"."nickname" IS 'Full name';

COMMENT ON COLUMN "db_organization"."id" IS 'Primary key of organization';

COMMENT ON COLUMN "db_organization"."code" IS 'Organization code';

COMMENT ON COLUMN "db_organization"."registration_number" IS 'Registration number of the organization with this system (not the tax id: that is a contact of type TAX_ID)';
COMMENT ON COLUMN "db_organization"."contact_group_id" IS 'Organization contacts (phone, fax, email, tax id ...)';

COMMENT ON COLUMN "db_organization"."active" IS 'Whether organization is active';
COMMENT ON COLUMN "db_organization"."image_key" IS 'Object-storage key of the logo (NULL = none)';

COMMENT ON COLUMN "db_organization_label"."organization_id" IS 'Referenced organization';

COMMENT ON COLUMN "db_organization_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_organization_label"."label" IS 'Localized organization name';

COMMENT ON COLUMN "db_company"."id" IS 'Primary key of company';

COMMENT ON COLUMN "db_company"."organization_id" IS 'Parent organization';

COMMENT ON COLUMN "db_company"."code" IS 'Company code';

COMMENT ON COLUMN "db_company"."master_lookup_id" IS 'Company type';

COMMENT ON COLUMN "db_company"."active" IS 'Whether company is active';
COMMENT ON COLUMN "db_company"."contact_group_id" IS 'Company contacts (phone, fax, email, tax id ...)';
COMMENT ON COLUMN "db_company"."image_key" IS 'Object-storage key of the logo (NULL = none)';

COMMENT ON COLUMN "db_company_label"."company_id" IS 'Referenced company';

COMMENT ON COLUMN "db_company_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_company_label"."label" IS 'Localized company name';

COMMENT ON COLUMN "db_branch"."id" IS 'Primary key of branch';

COMMENT ON COLUMN "db_branch"."organization_id" IS 'Owning organization';

COMMENT ON COLUMN "db_branch"."company_id" IS 'Owning company';

COMMENT ON COLUMN "db_branch"."code" IS 'Branch code';

COMMENT ON COLUMN "db_branch"."master_lookup_id" IS 'Branch type';

COMMENT ON COLUMN "db_branch"."address_group_id" IS 'Branch address group';

COMMENT ON COLUMN "db_branch"."contact_group_id" IS 'Branch contact group';

COMMENT ON COLUMN "db_branch"."active" IS 'Whether branch is active';

COMMENT ON COLUMN "db_branch_label"."branch_id" IS 'Referenced branch';

COMMENT ON COLUMN "db_branch_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_branch_label"."label" IS 'Localized branch name';

COMMENT ON COLUMN "db_department"."id" IS 'Primary key of department';

COMMENT ON COLUMN "db_department"."organization_id" IS 'Owning organization';

COMMENT ON COLUMN "db_department"."company_id" IS 'Owning company';

COMMENT ON COLUMN "db_department"."branch_id" IS 'Owning branch';

COMMENT ON COLUMN "db_department"."code" IS 'Department code';

COMMENT ON COLUMN "db_department"."active" IS 'Whether department is active';

COMMENT ON COLUMN "db_department_label"."department_id" IS 'Referenced department';

COMMENT ON COLUMN "db_department_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_department_label"."label" IS 'Localized department name';

COMMENT ON COLUMN "db_system"."id" IS 'Primary key of system';

COMMENT ON COLUMN "db_system"."code" IS 'System code';

COMMENT ON COLUMN "db_system"."active" IS 'Whether system is active';

COMMENT ON COLUMN "db_system_label"."system_id" IS 'Referenced system';

COMMENT ON COLUMN "db_system_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_system_label"."label" IS 'Localized system name';

COMMENT ON COLUMN "db_module"."id" IS 'Primary key of module';

COMMENT ON COLUMN "db_module"."system_id" IS 'Parent system';

COMMENT ON COLUMN "db_module"."code" IS 'Module code';

COMMENT ON COLUMN "db_module"."active" IS 'Whether module is active';

COMMENT ON COLUMN "db_module_label"."module_id" IS 'Referenced module';

COMMENT ON COLUMN "db_module_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_module_label"."label" IS 'Localized module name';

COMMENT ON COLUMN "db_program"."id" IS 'Primary key of program';

COMMENT ON COLUMN "db_program"."system_id" IS 'Parent system';

COMMENT ON COLUMN "db_program"."module_id" IS 'Parent module';

COMMENT ON COLUMN "db_program"."program_id" IS 'Parent program (for hierarchy)';

COMMENT ON COLUMN "db_program"."code" IS 'program code';

COMMENT ON COLUMN "db_program"."path" IS 'Routing path';

COMMENT ON COLUMN "db_program"."active" IS 'Whether program is active';

COMMENT ON COLUMN "db_program_label"."program_id" IS 'Referenced program';

COMMENT ON COLUMN "db_program_label"."language_id" IS 'Language of label';

COMMENT ON COLUMN "db_program_label"."label" IS 'Localized program name';

COMMENT ON COLUMN "su_user"."must_change_password" IS 'The account may not sign in until its password is (re)set through the emailed link';
COMMENT ON COLUMN "su_configuration"."id" IS 'Primary key of system configuration';

COMMENT ON COLUMN "su_configuration"."category" IS 'Configuration category';

COMMENT ON COLUMN "su_configuration"."key" IS 'Configuration key';

COMMENT ON COLUMN "su_configuration"."value" IS 'Configuration value';

COMMENT ON COLUMN "su_configuration"."description" IS 'Configuration description';

COMMENT ON COLUMN "su_permission_group"."id" IS 'Primary key of permission group';

COMMENT ON COLUMN "su_permission_group"."code" IS 'Permission group code';

COMMENT ON COLUMN "su_permission_group"."active" IS 'Whether permission group is active';

COMMENT ON COLUMN "su_permission"."id" IS 'Primary key of permission';

COMMENT ON COLUMN "su_permission"."permission_group_id" IS 'Owning permission group';

COMMENT ON COLUMN "su_permission"."system_id" IS 'Target system';

COMMENT ON COLUMN "su_permission"."module_id" IS 'Target module';

COMMENT ON COLUMN "su_permission"."program_id" IS 'Target program';

COMMENT ON COLUMN "su_permission"."master_lookup_id" IS 'Permission type (read, write, approve)';

COMMENT ON COLUMN "su_permission"."active" IS 'Whether permission is active';

COMMENT ON COLUMN "db_employee"."id" IS 'Primary key of employee';

COMMENT ON COLUMN "db_employee"."organization_id" IS 'Employee organization';

COMMENT ON COLUMN "db_employee"."company_id" IS 'Employee company';

COMMENT ON COLUMN "db_employee"."branch_id" IS 'Employee branch';

COMMENT ON COLUMN "db_employee"."department_id" IS 'Employee department';

COMMENT ON COLUMN "db_employee"."person_name_id" IS 'Employee name';

COMMENT ON COLUMN "db_employee"."address_group_id" IS 'Employee address group';

COMMENT ON COLUMN "db_employee"."contact_group_id" IS 'Employee contact group';

COMMENT ON COLUMN "db_employee"."code" IS 'Employee code';

COMMENT ON COLUMN "db_employee"."active" IS 'Whether employee is active';

COMMENT ON COLUMN "su_user"."id" IS 'Primary key of system user';

COMMENT ON COLUMN "su_user"."employee_id" IS 'Linked employee';

COMMENT ON COLUMN "su_user"."username" IS 'Login username';

COMMENT ON COLUMN "su_user"."permission_group_id" IS 'Assigned permission group';

COMMENT ON COLUMN "su_user"."password_hash" IS 'Credential hash read by Casdoor''s Database syncer (mapped to Casdoor user.password, isHashed). Empty string means "no initial credential" - the syncer then leaves whatever Casdoor already has untouched';

COMMENT ON COLUMN "su_user"."password_type" IS 'Hash algorithm of password_hash, in Casdoor''s own vocabulary - one of the eight strings its cred.GetCredManager dispatches on (plain / salt / sha512-salt / md5-salt / bcrypt / pbkdf2-salt / argon2id / pbkdf2-django). Mapped to Casdoor user.passwordType, and must match the Casdoor organization''s own password type';

COMMENT ON COLUMN "su_user"."created_time" IS 'Account creation time, mapped to Casdoor user.createdTime by the Database syncer';

COMMENT ON COLUMN "su_user"."active" IS 'Whether user account is active';

ALTER TABLE "db_language_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_language_label" ADD FOREIGN KEY ("display_language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_currency_label" ADD FOREIGN KEY ("currency_id") REFERENCES "db_currency" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_currency_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_country_label" ADD FOREIGN KEY ("country_id") REFERENCES "db_country" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_country_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_province" ADD FOREIGN KEY ("country_id") REFERENCES "db_country" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_province_label" ADD FOREIGN KEY ("province_id") REFERENCES "db_province" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_province_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_district" ADD FOREIGN KEY ("country_id") REFERENCES "db_country" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_district" ADD FOREIGN KEY ("province_id") REFERENCES "db_province" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_district_label" ADD FOREIGN KEY ("district_id") REFERENCES "db_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_district_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_sub_district" ADD FOREIGN KEY ("country_id") REFERENCES "db_country" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_sub_district" ADD FOREIGN KEY ("province_id") REFERENCES "db_province" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_sub_district" ADD FOREIGN KEY ("district_id") REFERENCES "db_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_sub_district_label" ADD FOREIGN KEY ("sub_district_id") REFERENCES "db_sub_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_sub_district_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_postal_code" ADD FOREIGN KEY ("country_id") REFERENCES "db_country" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_postal_code" ADD FOREIGN KEY ("province_id") REFERENCES "db_province" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_postal_code" ADD FOREIGN KEY ("district_id") REFERENCES "db_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_postal_code" ADD FOREIGN KEY ("sub_district_id") REFERENCES "db_sub_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_master_lookup_label" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_master_lookup_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("address_group_id") REFERENCES "db_address_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("province_id") REFERENCES "db_province" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("district_id") REFERENCES "db_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("sub_district_id") REFERENCES "db_sub_district" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_address" ADD FOREIGN KEY ("postal_code_id") REFERENCES "db_postal_code" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_contact" ADD FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_contact" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_person_name" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_person_name_label" ADD FOREIGN KEY ("person_name_id") REFERENCES "db_person_name" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_person_name_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_organization_label" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_organization_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_company" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_company" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_company_label" ADD FOREIGN KEY ("company_id") REFERENCES "db_company" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_company_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch" ADD FOREIGN KEY ("company_id") REFERENCES "db_company" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch" ADD FOREIGN KEY ("address_group_id") REFERENCES "db_address_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_organization" ADD FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_company" ADD FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch" ADD FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch_label" ADD FOREIGN KEY ("branch_id") REFERENCES "db_branch" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_branch_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_department" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_department" ADD FOREIGN KEY ("company_id") REFERENCES "db_company" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_department" ADD FOREIGN KEY ("branch_id") REFERENCES "db_branch" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_department_label" ADD FOREIGN KEY ("department_id") REFERENCES "db_department" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_department_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_system_label" ADD FOREIGN KEY ("system_id") REFERENCES "db_system" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_system_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_module" ADD FOREIGN KEY ("system_id") REFERENCES "db_system" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_module_label" ADD FOREIGN KEY ("module_id") REFERENCES "db_module" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_module_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_program" ADD FOREIGN KEY ("system_id") REFERENCES "db_system" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_program" ADD FOREIGN KEY ("module_id") REFERENCES "db_module" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_program" ADD FOREIGN KEY ("program_id") REFERENCES "db_program" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_program_label" ADD FOREIGN KEY ("program_id") REFERENCES "db_program" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_program_label" ADD FOREIGN KEY ("language_id") REFERENCES "db_language" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_permission" ADD FOREIGN KEY ("permission_group_id") REFERENCES "su_permission_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_permission" ADD FOREIGN KEY ("system_id") REFERENCES "db_system" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_permission" ADD FOREIGN KEY ("module_id") REFERENCES "db_module" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_permission" ADD FOREIGN KEY ("program_id") REFERENCES "db_program" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_permission" ADD FOREIGN KEY ("master_lookup_id") REFERENCES "db_master_lookup" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("company_id") REFERENCES "db_company" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("branch_id") REFERENCES "db_branch" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("department_id") REFERENCES "db_department" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("person_name_id") REFERENCES "db_person_name" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("address_group_id") REFERENCES "db_address_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "db_employee" ADD FOREIGN KEY ("contact_group_id") REFERENCES "db_contact_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user" ADD FOREIGN KEY ("employee_id") REFERENCES "db_employee" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user" ADD FOREIGN KEY ("permission_group_id") REFERENCES "su_permission_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("user_id") REFERENCES "su_user" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("permission_group_id") REFERENCES "su_permission_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("organization_id") REFERENCES "db_organization" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("company_id") REFERENCES "db_company" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("branch_id") REFERENCES "db_branch" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "su_user_assignment" ADD FOREIGN KEY ("department_id") REFERENCES "db_department" ("id") DEFERRABLE INITIALLY IMMEDIATE;

COMMENT ON TABLE "su_user_assignment" IS 'Permission group a user holds at one scope (organization/company/branch/department); a user can hold several';

COMMENT ON COLUMN "su_user_assignment"."permission_group_id" IS 'Group whose programs/actions apply inside this scope';

------------------------------------------------------
-- Report jobs (RPPG000): what each user asked the report worker to build
------------------------------------------------------
-- api-master publishes the request to Redpanda (`report.requests`); the report worker inserts the
-- row when it picks the request up and updates it as the job runs: queued -> running ->
-- succeeded | failed. The file itself lives in object storage under `object_key`.
CREATE TABLE "rp_report_job" (
  "id" uuid PRIMARY KEY NOT NULL,
  "report_kind" varchar(50) NOT NULL,
  "requested_by" varchar(255) NOT NULL,
  "filter" jsonb NOT NULL DEFAULT '{}'::jsonb,
  "status" varchar(20) NOT NULL DEFAULT 'running',
  "row_count" bigint NOT NULL DEFAULT 0,
  "size_bytes" bigint NOT NULL DEFAULT 0,
  "file_name" varchar(255),
  "object_key" varchar(500),
  "error" text,
  "requested_at" timestamptz NOT NULL,
  "started_at" timestamptz,
  "finished_at" timestamptz,
  "row_version" bigint NOT NULL DEFAULT 1,
  CONSTRAINT "rp_report_job_status_check" CHECK (status IN ('queued', 'running', 'succeeded', 'failed'))
);

CREATE INDEX ON "rp_report_job" ("requested_by", "requested_at" DESC);

COMMENT ON TABLE "rp_report_job" IS 'Report requests and their outcome (file in object storage)';

------------------------------------------------------
-- Notifications (NTPG000): what the system tells each user
------------------------------------------------------
-- Any service publishes a notification to Redpanda (`notifications`); the notification service
-- stores it here (the id is the notification's own UUIDv7, so a redelivered message is a no-op)
-- and pushes it to the recipient's open sessions. `read_at` is when the user read it.
CREATE TABLE "nt_notification" (
  "id" uuid PRIMARY KEY NOT NULL,
  "recipient" varchar(255) NOT NULL,
  "kind" varchar(50) NOT NULL,
  "title" varchar(255) NOT NULL,
  "body" text NOT NULL DEFAULT '',
  "data" jsonb NOT NULL DEFAULT '{}'::jsonb,
  "link" varchar(500),
  "created_at" timestamptz NOT NULL,
  "read_at" timestamptz,
  "row_version" bigint NOT NULL DEFAULT 1
);

CREATE INDEX ON "nt_notification" ("recipient", "created_at" DESC);

COMMENT ON TABLE "nt_notification" IS 'Notifications per recipient (Kratos username); read_at NULL = unread';

------------------------------------------------------
-- Data-change capture (audit): who changed which row, before / after
------------------------------------------------------
-- Every INSERT / UPDATE / DELETE a user makes through the API leaves one row here, written by the
-- trigger below inside the same transaction as the change (so a rolled-back change leaves
-- nothing). The API shifts the rows to Quickwit (`kind=data_change`, page ALPG000) and deletes
-- them, so this table only ever holds what has not been shipped yet.
--
-- The actor comes from the session settings app.actor_id / app.actor_name / app.request_id which
-- the API sets on each connection for the duration of a write request (infrastructure
-- `audit_actor`). No actor = not a user action (seeds, migrations, workers) = nothing captured.
CREATE TABLE "al_data_change" (
  "id" uuid PRIMARY KEY NOT NULL DEFAULT encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  "changed_at" timestamptz NOT NULL DEFAULT clock_timestamp(),
  "txid" bigint NOT NULL DEFAULT txid_current(),
  "table_name" text NOT NULL,
  "op" text NOT NULL,
  "row_id" text,
  "actor_id" text NOT NULL,
  "actor_name" text,
  "request_id" text,
  "old_data" jsonb,
  "new_data" jsonb
);

CREATE INDEX "al_data_change_changed_at_idx" ON "al_data_change" ("changed_at");

COMMENT ON TABLE "al_data_change" IS 'Outbox of row changes made by API users, shipped to Quickwit and then deleted';

CREATE FUNCTION "al_capture_change"() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
  actor text := nullif(current_setting('app.actor_id', true), '');
  before_row jsonb;
  after_row jsonb;
BEGIN
  IF actor IS NULL THEN
    RETURN NULL;
  END IF;
  IF TG_OP <> 'INSERT' THEN before_row := to_jsonb(OLD); END IF;
  IF TG_OP <> 'DELETE' THEN after_row := to_jsonb(NEW); END IF;
  -- An UPDATE that changed nothing but the row_version bookkeeping column is not a change
  IF TG_OP = 'UPDATE' AND (before_row - 'row_version') = (after_row - 'row_version') THEN
    RETURN NULL;
  END IF;
  INSERT INTO "al_data_change" ("table_name", "op", "row_id", "actor_id", "actor_name", "request_id", "old_data", "new_data")
  VALUES (TG_TABLE_NAME, TG_OP, COALESCE(after_row, before_row) ->> 'id', actor,
          nullif(current_setting('app.actor_name', true), ''),
          nullif(current_setting('app.request_id', true), ''),
          before_row, after_row);
  RETURN NULL;
END
$$;

-- One trigger on every table of the schema except the audit outbox itself. A table added later
-- needs the same CREATE TRIGGER (or a re-run of this loop).
DO $$
DECLARE t text;
BEGIN
  FOR t IN SELECT tablename FROM pg_tables WHERE schemaname = current_schema() AND tablename NOT LIKE 'al\_%'
  LOOP
    EXECUTE format('CREATE TRIGGER "al_capture_change" AFTER INSERT OR UPDATE OR DELETE ON %I FOR EACH ROW EXECUTE FUNCTION "al_capture_change"()', t);
  END LOOP;
END
$$;
