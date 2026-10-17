-- Forced password change: an account with this flag cannot sign in until it has set a new password
-- through the emailed link (the identity provider's settings hook clears it). Idempotent;
-- sql/01_ysql.sql carries the same column for a fresh database.
ALTER TABLE "su_user" ADD COLUMN IF NOT EXISTS "must_change_password" boolean NOT NULL DEFAULT false;
COMMENT ON COLUMN "su_user"."must_change_password" IS 'The account may not sign in until its password is (re)set through the emailed link';
