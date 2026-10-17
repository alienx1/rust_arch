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
| `53_admin_seed.sql` | `ADMIN` group , every grant derived from `db_program`, employee, `admin` account | yes |
| `54_abac_test_seed.sql` | `ABAC_TEST_DEPT` (READ only, assigned at the IT department) + `test_abac` account | yes |
| `51_20261014_migrate_org_company_image.sql`, `51_20261014_migrate_employee_image.sql` | migration: คอลัมน์ `image_key` (โลโก้/รูปพนักงาน) ของ `db_organization` / `db_company` / `db_employee` | yes |
| `55`–`90` | UI dictionary (`db_label`): base labels, API error templates, per-page translations | yes |

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


## เปลี่ยน schema (ตาราง/คอลัมน์) — วิธีที่ใช้ได้จริงตอนนี้

ไม่มีเครื่องมือ migration แยก; สิ่งที่มีคือ **ลำดับไฟล์ + กฎของ `kube/arch/utils/db-seed.sh`**:

- ไฟล์เลข **≤ 50** (`00_`–`50_`) โหลด **ครั้งเดียว** ตอนฐานยังว่าง (ถ้าพบตาราง `db_language` แล้วจะข้าม)
- ไฟล์เลข **≥ 51** รันซ้ำ **ทุกครั้ง** ที่ `make seed` จึงต้องเป็น idempotent
- `01_ysql.sql` คือ "ความจริงของ schema" สำหรับฐานใหม่ — ต้องแก้ให้ตรงเสมอ

ดังนั้นการเปลี่ยน schema ทุกครั้งต้องทำ **สองที่คู่กัน**:

| ที่ | ทำอะไร | ใช้กับ |
|---|---|---|
| `sql/01_ysql.sql` | แก้ `CREATE TABLE` / เพิ่มคอลัมน์ / index ตามที่ต้องการให้เป็นในอนาคต | ฐานใหม่ (`make bootstrap`) |
| `sql/51_<YYYYMMDD>_migrate_<อะไร>.sql` | คำสั่ง **idempotent** ที่ทำให้ฐานเดิมไปถึงสถานะเดียวกัน | ฐานที่มีอยู่แล้ว (`make seed`) |

ชื่อไฟล์ migration ใช้รูปแบบ `51_20261015_migrate_add_foo.sql`: เลขนำ `51` ทำให้รันทุกครั้ง และเรียง (`sort -V`) ก่อน `51_org_chart_seed.sql` จึงเสร็จก่อน seed ที่อาจต้องใช้คอลัมน์ใหม่

ตัวอย่างคำสั่ง idempotent:

```sql
-- เพิ่มคอลัมน์
ALTER TABLE "db_branch" ADD COLUMN IF NOT EXISTS "foo" text;

-- เพิ่มตาราง (+ trigger audit: ตารางที่สร้างทีหลังบล็อกท้าย 01_ysql.sql ไม่ได้ trigger อัตโนมัติ)
CREATE TABLE IF NOT EXISTS "xx_bar" ( "id" uuid PRIMARY KEY NOT NULL, "row_version" bigint NOT NULL DEFAULT 1 );
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'al_capture_change' AND tgrelid = 'xx_bar'::regclass) THEN
    CREATE TRIGGER "al_capture_change" AFTER INSERT OR UPDATE OR DELETE ON "xx_bar"
      FOR EACH ROW EXECUTE FUNCTION "al_capture_change"();
  END IF;
END $$;

-- ค่าเริ่มต้น / เติมข้อมูลย้อนหลัง
UPDATE "db_branch" SET "foo" = '' WHERE "foo" IS NULL;
```

กติกา:

- **เพิ่มได้ปลอดภัย** (ตาราง/คอลัมน์ที่ nullable หรือมี default, index) — ทำแบบข้างบน
- **เปลี่ยน/ลบคอลัมน์, เปลี่ยนชนิด, เพิ่ม `NOT NULL` ลงข้อมูลเดิม** เป็นงานแบบ 2 ขั้น (deploy โค้ดที่รองรับทั้งสองแบบ → ย้ายข้อมูล → ค่อยลบของเก่า) อย่าทำในไฟล์ migration เดียวกับที่โค้ดเก่ายังใช้อยู่
- หลังแก้ schema: `make seed` → `make gen-entities` (ตรวจ diff ของ `domain/src/entity/`) → `make cache-flush` (seed เรียกให้เอง)
- ถ้าเป็นการเปลี่ยนที่ฐานเดิมรับไม่ได้จริง ๆ: เพิ่มชื่อตารางใหม่ใน guard ของ `db-seed.sh` เพื่อให้ฐานเก่าหยุดพร้อมข้อความ "recreate" (วิธีสุดท้าย: `make down && make bootstrap` **ลบข้อมูลทั้งหมด**)
- ก่อนใช้กับฐานจริง: สำรองข้อมูลและลองบนสำเนาก่อนเสมอ ([`PRODUCTION_CHECKLIST.md`](../docs/guides/PRODUCTION_CHECKLIST.md))
- ฐาน multi-region (pgEdge Spock): DDL ต้องทำให้ทุก node ตรงกัน — ตามแนวทางใน `docs/architecture/04_MULTI_REGION_TOPOLOGY.md`
