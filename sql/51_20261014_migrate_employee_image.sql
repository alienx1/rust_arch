-- Employee photo: the object-storage key of the uploaded image (NULL = none).
-- Idempotent; sql/01_ysql.sql carries the same column for a fresh database.
ALTER TABLE "db_employee" ADD COLUMN IF NOT EXISTS "image_key" varchar(500);
COMMENT ON COLUMN "db_employee"."image_key" IS 'Object-storage key of the photo (NULL = none)';
