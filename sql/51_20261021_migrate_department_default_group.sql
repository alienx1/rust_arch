-- A department's default permission group: the accounts of its employees get this group at the
-- department's branch, automatically (on account creation, and when the default is set or changed).
-- Idempotent; sql/01_ysql.sql carries the same column for a fresh database.

ALTER TABLE "db_department" ADD COLUMN IF NOT EXISTS "default_permission_group_id" uuid;
COMMENT ON COLUMN "db_department"."default_permission_group_id" IS 'Permission group every account of the department''s employees gets, at the department''s branch (added automatically)';

DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'db_department_default_permission_group_id_fkey') THEN
    ALTER TABLE "db_department" ADD CONSTRAINT "db_department_default_permission_group_id_fkey"
      FOREIGN KEY ("default_permission_group_id") REFERENCES "su_permission_group" ("id") DEFERRABLE INITIALLY IMMEDIATE;
  END IF;
END $$;
