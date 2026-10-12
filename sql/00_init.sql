-- 1. Create the main application database
SELECT 'CREATE DATABASE arch'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'arch')\gexec

-- 2. Connect to it
\c arch;

-- 3. Create arch schema for our application tables
CREATE SCHEMA IF NOT EXISTS arch;

-- 4. NOTE: search_path is intentionally NOT set at the database level here
-- (no `ALTER DATABASE arch SET search_path ...`). That would apply to every
-- connection to "arch", including Casdoor's (docker/compose/casdoor/app.conf
-- points its dataSourceName at this same "arch" database) — Casdoor doesn't
-- switch to a separate database for the postgres driver, it just creates
-- its own ~40 tables (casbin_rule, permission_rule, user, organization, ...)
-- unqualified in whatever schema search_path resolves to. A database-level
-- default search_path of "arch" would silently dump all of Casdoor's tables
-- into our own "arch" schema. Instead, docker-compose.yml's yugabyte-init
-- sets `search_path` per-session (`-c "SET search_path TO arch, public;"`)
-- only for the ysqlsh sessions that load design/database/ysql/*.sql, so
-- Casdoor's own connection keeps Postgres' true default ("public") and the
-- two stay fully separated.
