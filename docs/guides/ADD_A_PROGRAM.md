# คู่มือ: เพิ่มโปรแกรม (หน้าจอ + API) ใหม่ตั้งแต่ต้นจนจบ

ตัวอย่างอ้างอิงที่ใช้ไล่ดูโค้ดจริงได้: **ALPG001 Process Monitor** (อ่านตาราง + ปุ่มสั่งงาน, มี test ครบ) และ **DBPG005 Geography** (CRUD เต็มรูป)
สมมติจะเพิ่มโปรแกรม `XXPGNNN` ในระบบ `XX` — แทนที่ชื่อด้วยของจริงตลอดคู่มือ

> กฎที่ห้ามละเมิดอยู่ที่ [`AGENTS.md`](../../AGENTS.md) (ชั้น dependency, ห้ามชื่อไฟล์ generic เช่น `service.rs`/`handler.rs`/`utils.rs`, test อยู่ใน `tests/`, UUIDv7, `{param}` ของ Axum 0.8)

## 0. ตัดสินใจก่อนเขียน

| เรื่อง | ค่าที่ต้องเลือก |
|---|---|
| รหัสโปรแกรม | `XXPGNNN` — `XX` = ระบบ 2 ตัวอักษร, `PG` = page, `NNN` = เลขลำดับ (ใหญ่/ตัวพิมพ์ใหญ่) |
| path เว็บ | `/xx/xxpgnnn` (ตัวพิมพ์เล็ก — เมนูใช้ค่านี้เป็นเส้นทางหลักของโปรแกรม) |
| path API | `/api/v1/xx/xxpgnnn/...` — **ต้องมี segment `xxpgnnn`** เพราะ audit หา "โปรแกรม" ของ request จาก segment นี้ (`infrastructure/src/middleware/audit_target.rs`) |
| สิทธิ์ | `READ` (ดู), `WRITE` (เพิ่ม/แก้/ลบ/สั่งงาน), `APPROVE` (อนุมัติ) — ตรวจที่ router ด้วย `require(&access, &user, "XXPGNNN", PermissionAction::…)` |
| ขอบเขตข้อมูล | ถ้างานผูกกับองค์กร/บริษัท/สาขา/แผนก ใช้ `require_in` + `ScopeTarget` (ดู DBPG003 / `access_scope_test.rs`) |
| เมนู | โปรแกรมอยู่ใต้โฟลเดอร์ (เช่น `ALPG000_GROUP`) หรืออยู่เดี่ยว; ถ้าเป็นหน้าของ "ตัวผู้ใช้เอง" (รายงานของฉัน, แจ้งเตือน) ให้เข้าเมนู dropdown ผู้ใช้ — ดู `USER_MENU_PROGRAMS` ใน `layouts/master_navigation.rs` |

## 1. ฐานข้อมูล

1. เพิ่มตารางใน [`sql/01_ysql.sql`](../../sql/README.md) — ชื่อ `xx_<ชื่อ>`, PK เป็น `uuid` (สร้างด้วย **UUIDv7** ที่ฝั่ง Rust: `Uuid::now_v7()`), มี `row_version bigint NOT NULL DEFAULT 1` (optimistic locking) ถ้าแก้ไขได้
   - วางตาราง **ก่อน** บล็อก `DO $$ … al_capture_change …` ท้ายไฟล์ → ได้ trigger เก็บ before/after ลง audit ให้อัตโนมัติ
   - ตารางที่เพิ่มทีหลังใน migration ต้อง `CREATE TRIGGER "al_capture_change" …` เอง (ดู [`sql/README.md`](../../sql/README.md) หัวข้อ "เปลี่ยน schema")
2. ถ้าตารางสำคัญจนเมื่อไม่มีแล้วต้องบังคับสร้างฐานใหม่ ให้เพิ่มชื่อตารางใน guard ของ `kube/arch/utils/db-seed.sh`
3. สร้าง entity: ฐานต้องมีตารางแล้ว (`make seed` หรือ `make bootstrap`) แล้วรัน `make gen-entities` → ไฟล์ `domain/src/entity/xx_<ชื่อ>.rs` + `mod.rs`/`prelude.rs`
   - คำสั่งนี้ **สร้างทุก entity ใหม่ทั้งหมด**: ดู diff ทุกครั้ง อย่าให้ไฟล์เดิมที่ปรับมือถูกเขียนทับ

## 2. ชั้น application — `application/master/src/features/xx/xxpgnnn/`

```text
features/xx/xxpgnnn/
├── mod.rs                      # re-export + คำอธิบายโปรแกรม (รหัส, API, แหล่งข้อมูล)
├── errors/mod.rs               # enum Xxpgnnn Error (thiserror): Validation / NotFound / Conflict / Unavailable / Internal
├── models/                     # DTO + filter query + (re-export entity เป็น *Model)
├── commands/
│   ├── mod.rs                  # trait XxpgnnnCommands  ← router เรียกผ่านตัวนี้เท่านั้น
│   ├── xxpgnnn_executor.rs     # struct …CommandExecutor ที่ impl trait (บาง ๆ: เรียก <action>.rs)
│   └── <action>_<target>.rs    # ตรรกะแต่ละ action (list_processes.rs, reprocess_process.rs …)
└── queries/
    ├── mod.rs                  # trait XxpgnnnQueries
    └── db_xxpgnnn_queries.rs   # Sea-ORM ตรง ๆ (ไม่ต้องมี repository เพิ่ม)
```

- ลงทะเบียน module ใน `features/xx/mod.rs` (`pub mod xxpgnnn;`) — ถ้าต้องการ re-export ที่ `features/mod.rs` ด้วย
- กฎการทำงาน: validate/normalize ใน `commands` (ดู `filter_rules.rs`), query ล้วนอยู่ใน `queries`, **ห้าม import `handler/*` หรือ driver ตรง ๆ** — ต้องใช้ port จาก `application-core` (เช่น `EventService`, `CacheService`)
- ทำให้ทดสอบง่าย: executor รับ `Arc<dyn …Queries>` (+ port อื่นผ่าน `with_xxx(...)`) — ใน test ใช้ mock ในหน่วยความจำ
- ถ้าอ่านข้อมูลที่ "อ่านบ่อยแก้น้อย" ให้ห่อด้วยแคช (หัวข้อ 6)

## 3. ชั้น API — `handler/api/master/`

1. `src/routers/xxpgnnn_router.rs` — สร้างตาม `alpg001_router.rs`:
   - `pub const XXPGNNN_PATH: &str = "/xx/xxpgnnn";` และ `const PROGRAM: &str = "XXPGNNN";`
   - handler รับ `State(commands): State<Arc<dyn XxpgnnnCommands>>` + `State(access): State<Arc<dyn AccessControl>>` + `Extension(user)` + `wire: WireFormat` → ตรวจ `require(...)` เป็นอย่างแรก → ตอบด้วย `wire.respond(...)` (เจรจา JSON/CBOR ให้เอง)
   - `ApiError` ที่ `impl IntoResponse` แปลง error ของ feature เป็น HTTP (400/404/409/503/500) ด้วยรูปแบบ `{success, code, message}`
   - ส่งออก `xxpgnnn_routes()` และ `xxpgnnn_routes_with_state(state)` (ตัวหลังไว้ให้ test)
   - route แบบมีพารามิเตอร์ใช้ `/{id}` (Axum 0.8) ไม่ใช่ `/:id`
2. ลงทะเบียน: `routers/mod.rs` (`pub mod` + `pub use`), `routers/base_router.rs` (**ต้อง `.nest` สองที่**: ใน `api_router` และใน root router ที่ขึ้นต้น `/api/v1`)
3. `connection.rs`: เพิ่มฟิลด์ `xxpgnnn_commands: Arc<dyn XxpgnnnCommands>`, สร้าง executor (ต่อ `.with_cache(caches.clone())` ถ้ามีแคช), ใส่ใน `Self { … }` และเพิ่ม `impl FromRef<AppConnection> for Arc<dyn XxpgnnnCommands>`
4. การเขียนข้อมูล: ใช้ `actor.user_id` เป็น `change_by`; trigger audit จะจับ before/after เองเมื่อ API ตั้ง actor ไว้ (`infrastructure::track_actor`)
5. ถ้ามีการเขียนข้อมูลที่ถูกแคชไว้ที่อื่น → เพิ่มกฎใน `src/cache_policy.rs` (ข้อ 6)

## 4. Test (ห้ามเขียนใน `src/`)

| ที่ไหน | ทดสอบอะไร |
|---|---|
| `application/master/tests/xxpgnnn_test.rs` | กฎ validate/normalize, ผลของ executor บน mock queries, กรณีผิดพลาด |
| `handler/api/master/tests/xxpgnnn_router_test.rs` | สถานะ HTTP: 200/202, 400, 403 (ไม่มีสิทธิ์/มีแค่ READ), 404, 409, 503 โดยใช้ `…_routes_with_state` + `MockAccess` |

รัน: `cargo test -p application-master --test xxpgnnn_test` และ `cargo test -p api-master --test xxpgnnn_router_test`
(ตัวอย่างครบ: `alpg001_test.rs`, `alpg001_router_test.rs`)

## 5. Seed — โปรแกรมเข้าเมนู + สิทธิ์ + ข้อความ

1. **`sql/52_program_catalogue.sql`**: เพิ่มแถว `db_program` (id คงที่ รูปแบบ `019ca1ed-0000-7000-8003-…`, `code`, `path`, `icon`, `sort_order`, `program_id` = โฟลเดอร์แม่) และชื่อ th/en ใน `db_program_label`
   - ชื่อ `icon` ต้องมีใน `handler/web/package/src/layout/nav_icons.rs` (ไม่มี → ขึ้นเป็นจุด) เพิ่มได้ที่ตารางนั้น
2. **สิทธิ์**: ไม่ต้องเขียน — `sql/53_admin_seed.sql` แจก READ/WRITE/APPROVE ให้กลุ่ม ADMIN **ทุกโปรแกรมที่ active** ทุกครั้งที่รัน; กลุ่มอื่นกำหนดที่หน้า SUPG003 (permission matrix)
3. **ข้อความหน้าจอ**: ไฟล์ใหม่ `sql/NN_i18n_xxpgnnn.sql` (NN = เลขถัดไป, idempotent ด้วย `ON CONFLICT DO NOTHING`, id แบบ v7 — คัดลอกบล็อก `INSERT … SELECT` จาก `sql/80_i18n_alpg001.sql`) ใส่ key `xxpgnnn.*` ทั้ง `th` และ `en`
4. `make seed` แล้วเช็ก: `SELECT code FROM arch.db_program WHERE code='XXPGNNN'`

## 6. แคชและการล้างแคช (เมื่อเกี่ยวข้อง)

- อ่านบ่อย/แก้น้อย/ทุกคนเห็นเหมือนกัน → ห่อการอ่านใน executor: `caches.ns(cache_ns::REF).through_key(key, cache_ttl::REF, || …)` (ตัวอย่าง `dbpg005_executor.rs`)
- เป็นข้อมูลที่เปลี่ยนบ่อย/ขึ้นกับผู้ใช้/ต้องสดเสมอ → **ไม่แคช**
- โปรแกรมที่เขียนข้อมูลซึ่งมี namespace ไหนแคชไว้ → เพิ่มบรรทัดใน `handler/api/master/src/cache_policy.rs` (มี test กันลืมที่ `cache_policy_test.rs`)
- รายละเอียด: [`docs/architecture/11_VALKEY_READ_CACHE.md`](../architecture/11_VALKEY_READ_CACHE.md)

## 7. ฝั่งเว็บ — `handler/web/application/master/src/pages/xx/xxpgnnn/`

```text
pages/xx/xxpgnnn/
├── mod.rs                    # pub use screens::Xxpgnnn Screen as Xxpgnnn Page;
├── models/xxpgnnn_dto.rs     # DTO (Clone, Debug, PartialEq, Serialize, Deserialize)
├── services/xxpgnnn_service.rs      # เรียก HTTP (HttpClient) — แปลง 403/409/503 เป็นข้อความ
├── providers/xxpgnnn_context.rs     # context (Leptos signals) + hook use_xxpgnnn()
├── components/<widget>.rs           # UI ย่อยของหน้านี้
└── screens/xxpgnnn_screen.rs        # ประกอบหน้า: PageContainer/PageHeader + provide_…_context()
```

- **ตารางรายการใช้ `web_package::ui::component::DataTable` เสมอ** (ค้นหา, รีเฟรช, ตัวกรอง, แบ่งหน้า) — ห้ามสร้าง Card+Table เอง
- ลงทะเบียน: `pages/xx/mod.rs`, `pages/mod.rs` (`pub use …Page`), `routes/master_routes.rs` (`use` + `<Route path=path!("xx/xxpgnnn") view=XxpgnnnPage />`)
- ข้อความทุกจุดใช้ `t_or("xxpgnnn.key", "ข้อความไทยสำรอง")` ให้ตรงกับ key ใน seed i18n
- ปุ่มที่ต้องมีสิทธิ์: ซ่อน/ปิดตาม `use_access()` (แอปรู้สิทธิ์จาก `/api/v1/access/me`) แต่ **API ตรวจซ้ำเสมอ** — ห้ามเชื่อแค่ฝั่งเว็บ
- ช่วงวันที่ใช้ `DateRangePicker`, dropdown ใช้ `dismiss_on_outside` (อย่าทำชั้นโปร่งใสเต็มจอ: ใน topbar มี `backdrop-blur` ทำให้ `fixed` คลุมแค่ topbar)
- ดีไซน์: ใช้โทเคนสีของธีม (`bg-primary`, `text-muted-foreground`, `border-border` …) เพื่อให้ dark mode ทำงาน

## 8. เอกสาร

- `docs/programs/XXPGNNN_<ชื่อ>.md` (เป้าหมาย, API, สถานะ, ข้อจำกัด) + แถวใน `docs/programs/00_INDEX.md`

## 9. เช็กลิสต์ก่อนส่ง

- [ ] `make seed` ผ่าน และเห็นโปรแกรมในเมนู (ล็อกอิน admin)
- [ ] `cargo test --workspace --exclude master-app` ผ่าน, ไม่มี warning ใหม่
- [ ] `cargo check -p master-app --target wasm32-unknown-unknown` ผ่าน
- [ ] ลองในเบราว์เซอร์จริง: สถานะโหลด/ว่าง/ผิดพลาด, ไม่มีสิทธิ์ (ใช้ `test_abac` ที่มีแค่ READ), ภาษาไทย/อังกฤษ, dark mode, หน้าจอแคบ
- [ ] ดู audit ที่ ALPG000 ว่า API call + data change ของโปรแกรมใหม่ปรากฏ
- [ ] เขียนสเปกโปรแกรมใน `docs/programs/` แล้ว

## 10. กับดักที่เจอมาแล้วในโปรเจกต์นี้

| อาการ | สาเหตุ / วิธีแก้ |
|---|---|
| เมนูไม่ขึ้น | ลืม `make seed`; หรือไม่มี READ ในกลุ่มของผู้ใช้; หรือโปรแกรม `active=false` (เมนูถูกแคช 60 วินาที — `make cache-flush`) |
| API 404 ทั้งที่ลงทะเบียนแล้ว | ลืม `.nest` ตัวที่สองของ root router (`/api/v1…`); ผ่าน Trunk proxy ให้ใช้ path ที่ขึ้นต้น `/api/v1/` |
| route `/` ของ router ย่อยโดน proxy ตีความผิด | ตั้งเป็น `/items` แทน `/` (เจอกับ api-notification) |
| คลาส `bg-primary/10`, `border-border/60` ไม่แสดงสี | สีธีมเป็นตัวแปร `var(--…)` ที่ Tailwind 3 ใส่ alpha ให้ไม่ได้ → ใช้ `bg-[color-mix(in_oklch,var(--primary)_12%,transparent)]` |
| คลาส Tailwind ที่ประกอบด้วย `format!("w-{n}")` ไม่มีผล | CSS ถูกคอมไพล์จากข้อความที่ปรากฏใน `.rs` ตรง ๆ: ต้องเขียนชื่อคลาสเต็ม (`"w-4"`, `"w-8"`) หรือใส่ใน `safelist` ของ `tailwind.config.js` |
| เว็บยังเป็นของเก่า | trunk ยัง build ไม่เสร็จ (wasm ใหญ่) หรือแคชเบราว์เซอร์ — hard reload |
| เพิ่ม `web-sys` API แล้วคอมไพล์ไม่ผ่าน | ต้องเปิด feature ใน `Cargo.toml` ของ `handler/web/package` และ `handler/web/application/master` |
| `row_version` ไม่ตรง (409/400) | ฟอร์มแก้ไขต้องส่ง `row_version` ที่อ่านมาล่าสุดกลับ |
