-- Organization / company logo: the object-storage key of the uploaded image (NULL = none).
-- Idempotent; sql/01_ysql.sql carries the same columns for a fresh database.
ALTER TABLE "db_organization" ADD COLUMN IF NOT EXISTS "image_key" varchar(500);
ALTER TABLE "db_company" ADD COLUMN IF NOT EXISTS "image_key" varchar(500);
COMMENT ON COLUMN "db_organization"."image_key" IS 'Object-storage key of the logo (NULL = none)';
COMMENT ON COLUMN "db_company"."image_key" IS 'Object-storage key of the logo (NULL = none)';
