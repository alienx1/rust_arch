# sql/ — schema and seed data

`make bootstrap` (or `make seed` on a running cluster) runs every `*.sql` here **in file-name
order** (`sort -V`) through `kube/arch/utils/db-seed.sh`. A fresh database needs nothing else:
after it, `admin@arch.local` / `Admin@2026` can sign in (the Kratos identity is created by
`bootstrap.sh`).

| Files | What | Re-runnable |
|-------|------|-------------|
| `00_init.sql` | database + `arch` schema | load once |
| `01_ysql.sql` | the whole schema (maintained by hand — edit it directly; it is no longer generated from DBML) | load once |
| `02`–`50` | reference data: languages, currencies, countries, lookups, provinces … postal codes | load once |
| `51_org_chart_seed.sql` | starting organization → company → branch → department | yes |
| `52_program_catalogue.sql` | systems, modules and programs (the sidebar menu); no permissions | yes |
| `53_admin_seed.sql` | `ADMIN` group, every grant derived from `db_program`, employee, `admin` account | yes |
| `54_abac_test_seed.sql` | `ABAC_TEST_DEPT` (READ only, assigned at the IT department) + `test_abac` account | yes |
| `51_20261014_migrate_org_company_image.sql`, `51_20261014_migrate_employee_image.sql` | migration: `image_key` column (logo / employee photo) on `db_organization` / `db_company` / `db_employee` | yes |
| `55`–`94` | UI dictionary (`db_label`): base labels, API error templates, per-page translations | yes |

Order rules:

- A file only depends on files with a lower number. `53_`/`54_` need `51_` and `52_`; the
  label files need only the languages from `02_`.
- Label files use `ON CONFLICT DO NOTHING` (the first file to define a key wins) except
  `56_db_label_seed_all.sql`, which refreshes its values.
- A schema change goes into `01_ysql.sql` directly. Because it is load-once, an existing
  database must be recreated (`make down && make bootstrap`); `db-seed.sh` stops with that hint
  when it finds an older schema.
- New seed → next free number, idempotent (`ON CONFLICT`), ids as UUIDv7 (PostgreSQL 17 has no
  `uuidv7()`: see the generator expression in `75_i18n_table_page_size.sql`).


## Changing the Schema (Tables / Columns) — Current Workflow

There is no separate migration tool; schema evolution relies on **file ordering + the rules in `kube/arch/utils/db-seed.sh`**:

- Files numbered **≤ 50** (`00_`–`50_`) are loaded **once** when the database is empty (skipped if `db_language` already exists).
- Files numbered **≥ 51** run **every time** `make seed` is executed and must therefore be idempotent.
- `01_ysql.sql` is the single source of truth for fresh databases — always keep it up to date.

Every schema change must therefore be applied in **two places together**:

| Location | Action | Applies To |
|---|---|---|
| `sql/01_ysql.sql` | Update `CREATE TABLE` / add columns / add indexes to reflect the desired target state | Fresh databases (`make bootstrap`) |
| `sql/51_<YYYYMMDD>_migrate_<what>.sql` | Write **idempotent** statements that bring an existing database to the same state | Existing databases (`make seed`) |

Name migration files using the pattern `51_20261015_migrate_add_foo.sql`: the `51` prefix ensures it runs on every `make seed`, and `sort -V` places it before `51_org_chart_seed.sql` so migrations finish before any seed file that may depend on new columns.

Idempotent migration examples:

```sql
-- Add a column
ALTER TABLE "db_branch" ADD COLUMN IF NOT EXISTS "foo" text;

-- Add a table (+ audit trigger: tables created after the DO block at the end of 01_ysql.sql do not get the trigger automatically)
CREATE TABLE IF NOT EXISTS "xx_bar" ( "id" uuid PRIMARY KEY NOT NULL, "row_version" bigint NOT NULL DEFAULT 1 );
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'al_capture_change' AND tgrelid = 'xx_bar'::regclass) THEN
    CREATE TRIGGER "al_capture_change" AFTER INSERT OR UPDATE OR DELETE ON "xx_bar"
      FOR EACH ROW EXECUTE FUNCTION "al_capture_change"();
  END IF;
END $$;

-- Backfill / default values
UPDATE "db_branch" SET "foo" = '' WHERE "foo" IS NULL;
```

Guidelines:

- **Safe additive changes** (new tables, nullable columns, columns with defaults, indexes) — apply directly as shown above.
- **Renaming/dropping columns, changing types, or adding `NOT NULL` to existing data** requires a two-step rollout (deploy code compatible with both old and new schemas → migrate data → remove old schema in a later change). Do not break running code in a single migration step.
- After modifying the schema: `make seed` → `make gen-entities` (inspect `git diff` on `domain/src/entity/`) → `make cache-flush` (`make seed` runs this automatically).
- If a change is truly incompatible with existing databases: add the new table name to the guard in `db-seed.sh` so older databases stop with a "recreate" message (last resort: `make down && make bootstrap`, which **deletes all data**).
- Before running against a production database: always back up and test on a copy first ([`PRODUCTION_CHECKLIST.md`](../docs/guides/PRODUCTION_CHECKLIST.md)).
- Multi-region databases (pgEdge Spock): DDL must be kept consistent across all nodes — follow the guidelines in `docs/architecture/04_MULTI_REGION_TOPOLOGY.md`.
