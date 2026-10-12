# คู่มือ: เพิ่ม service ใหม่ (worker อ่าน Redpanda และ/หรือ API แยก)

ต้นแบบที่มีอยู่แล้วและให้ไล่ดูโค้ดได้:

| service | หน้าที่ | โค้ด |
|---|---|---|
| **api-report** (:8001) | worker อ่าน `report.requests` → สร้างไฟล์ CSV → Garage; API ดู/ดาวน์โหลดรายงาน | `application/report`, `handler/api/report` |
| **api-notification** (:8002) | อ่าน `notifications` → เก็บ DB → ผลักเข้าเบราว์เซอร์ทาง SSE | `application/notification`, `handler/api/notification` |
| **api-master** (:8000) | API หลักของข้อมูล master / สิทธิ์ / audit | `application/master`, `handler/api/master` |

## 0. ควรแยก service หรือเพิ่มใน api-master?

| แยกเป็น service ใหม่เมื่อ | เพิ่มใน api-master เมื่อ |
|---|---|
| มีงานเบื้องหลังยาว (สร้างไฟล์, ส่งอีเมล, ประมวลผลชุดใหญ่) | เป็น CRUD / อ่านเขียนข้อมูลปกติ |
| ต้องสเกล/restart แยกจาก API หลัก | ใช้ตารางและสิทธิ์ชุดเดียวกับโปรแกรมที่มีอยู่ |
| ถือ connection ค้าง (SSE, WebSocket) | request สั้น ๆ |
| ล้มแล้วต้องไม่ลาก API หลักล้มตาม | |

กฎเหล็ก: **service คุยกันผ่าน Redpanda (หรือฐานข้อมูล) ไม่เรียก HTTP หากัน** และส่ง "ข้อความ" ที่มี id ของตัวเองเสมอ

## 1. สัญญากลาง — `application/core/src/traits/<ชื่อ>.rs`

```rust
pub const MY_TOPIC: &str = "my.requests";          // ชื่อ topic
pub const MY_WORKER_GROUP: &str = "my-worker";     // consumer group

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MyMessage { pub job_id: String, /* ข้อมูลที่ worker ต้องใช้ครบในตัว */ }
```

- ข้อความต้อง **อ่านได้ด้วยตัวเอง** (ไม่ต้องกลับไปถามผู้ส่ง) และมี id (UUIDv7) ที่ใช้เป็น Kafka key
- เพิ่มฟิลด์ใหม่ภายหลังต้อง `#[serde(default)]` เพื่อให้ข้อความเก่าใน topic ยังอ่านได้ (ดู `reprocess` ใน `ReportJobMessage`)
- export ใน `traits/mod.rs`
- ฝั่งผู้ส่ง (เช่น api-master): `EventService::publish(topic, Some(&id), &bytes)` — ถ้า broker ล่มให้ตอบ 503 ไม่ใช่ทำเป็นสำเร็จ

## 2. use case — `application/<svc>/`

```text
application/<svc>/
├── Cargo.toml                  # name = "application-<svc>"  (เพิ่มใน [workspace] members + [workspace.dependencies] ที่ root)
└── src/features/<program>/     # โครงเดียวกับ ADD_A_PROGRAM (commands/queries/models/errors)
    └── commands/<x>_handler.rs # struct …Handler: impl EventHandler  → parse JSON → เรียก executor
```

- `EventHandler::handle(key, payload)`: **ห้าม panic และห้ามคืน error** — consumer commit offset หลัง `handle` จบเสมอ (at-least-once, ไม่มี poison message) ดังนั้น handler ต้อง **บันทึกความล้มเหลวของตัวเอง** (แถวสถานะ `failed` + ข้อความ) และ **idempotent** (ข้อความมาซ้ำ = ไม่ทำซ้ำ: ดู `run_report_job.rs` ที่ข้ามงานที่จบแล้ว)
- ข้อความที่อ่านไม่ออก: log แล้วข้าม (อ่านซ้ำกี่ครั้งก็ไม่ออก)
- ทดสอบด้วย mock ของ port ทั้งหมด (`application/report/tests/rppg000_test.rs` ใช้ storage/audit/notifier ในหน่วยความจำ)
- อยากแจ้งผู้ใช้: ใช้ `NotificationService::notify(username, Notification{…})` (ส่งเข้า topic `notifications` ให้ api-notification เก็บและผลัก SSE; ยิงแล้วลืม ไม่บล็อกงาน)

## 3. binary — `handler/api/<svc>/`

คัดจาก `handler/api/report/` (มี `[lib]` + `src/lib.rs` เพื่อให้ `tests/` import ได้):

| ไฟล์ | ทำอะไร |
|---|---|
| `Cargo.toml` | `name = "api-<svc>"`, `[lib] name = "api_<svc>"`, `[[bin]]`; เพิ่ม member ที่ root |
| `src/main.rs` | โหลด `.env`, tracing, อ่าน `<SVC>_HOST/PORT/PREFIX`, **graceful shutdown + watchdog 10 วินาที + Ctrl+C ซ้ำออกทันที** (กัน SSE/connection ค้างทำให้ปิดไม่ลง) |
| `src/connection.rs` | ต่อ DB / Valkey / Kratos / adapter ที่ต้องใช้; ประกอบ executor; `spawn_worker` |
| `src/routers/base_router.rs` | `/health`, `/livez`, `/readyz`; routes ของโปรแกรมครอบด้วย `require_kratos_session`; `cbor_error_responses`; CORS; trace |
| `src/routers/<program>_router.rs` | router ที่รับ `State<Arc<dyn …Commands>>` เท่านั้น |

worker อ่านข้อความ (ต้นแบบ `spawn_report_worker`):

```rust
tokio::spawn(async move {
    let consumer = RedpandaEventConsumer::new(&brokers);          // commit หลัง handler จบ
    let handler: Arc<dyn EventHandler> = Arc::new(MyHandler::new(commands));
    loop {
        if let Err(e) = consumer.consume(MY_TOPIC, MY_WORKER_GROUP, handler.clone()).await {
            error!(error = %e, "worker stopped, retrying in 5 s");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;         // ต่อใหม่เมื่อ broker หลุด
    }
});
```

**Consumer group:** หลาย instance ที่ "แบ่งงานกัน" ใช้ group เดียวกัน (ข้อความหนึ่งถูกทำโดยตัวเดียว); instance ที่ "ต้องเห็นทุกข้อความ" (เช่น ผลัก SSE ถึงทุก session) ต้องมี group ของตัวเองต่อ instance (ดู `NOTIFICATION_CONSUMER_GROUP`)

## 4. ต่อเข้ากับระบบ

| งาน | ที่ไหน |
|---|---|
| รันตอน dev | `Makefile`: เพิ่ม `run-<svc>` / alias / (`dev-<svc>`) และใส่ใน `.PHONY` |
| ตัวแปรแวดล้อม | `.env` และ `.env.example`: `<SVC>_HOST`, `<SVC>_PORT`, `<SVC>_PREFIX`, (`<SVC>_CONSUMER_GROUP`) |
| topic | `Makefile` target `topics`: เพิ่มชื่อ topic ใน loop (รันซ้ำได้) — `make bootstrap` **ไม่** สร้าง topic ให้ |
| เว็บเรียก service นี้ | `handler/web/application/master/Trunk.toml.tmpl` (**ไม่ใช่** `Trunk.toml` ซึ่งถูกสร้างจากเทมเพลต + `.env` ตอน `make web`): เพิ่ม `[[proxy]]` ของ prefix `/api/v1/<svc>/` → `@@<SVC>_URL@@@@<SVC>_PREFIX@@/` **ก่อน** รายการ `/api/v1/` (ตัวที่ยาวกว่าต้องมาก่อน) แล้วเพิ่มตัวแปรใน `gen-trunk-config.sh` และ `.env.example` |
| พอร์ต/ความปลอดภัย | `docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md` |
| ดู event ใน ALPG000 | เพิ่มดัชนี Quickwit ของ topic (ไฟล์ใน `kube/arch/applications/arch_logging/indexes/` + ชื่อใน `make event-indexes`) แล้วเพิ่ม kind ใน `audit_kind` (ดู `event_stream_query.rs`) — ทำเมื่อต้องการค้นอีเวนต์ของ topic |
| มอนิเตอร์/สั่งซ้ำ | ถ้าสถานะงานอยู่ในตาราง ให้ดูว่าควรให้ ALPG001 (Process Monitor) แสดงด้วยหรือไม่ |

> ใน dev ทุก service รัน **บนเครื่อง** (cargo) ส่วน Postgres/Valkey/Redpanda/Kratos/Garage/Quickwit อยู่ใน k3d
> repo นี้ **ยังไม่มี Dockerfile / manifest ของ service ตัวเอง** — ถ้าจะ deploy ใน cluster ต้องเพิ่มเอง (ดู [`PRODUCTION_CHECKLIST.md`](./PRODUCTION_CHECKLIST.md))

## 5. Infrastructure ใหม่ (เมื่อต้องใช้เทคโนโลยีใหม่)

1. ประกาศ **port (trait)** ใน `application/core/src/traits/` — ภาษาของโดเมน ไม่ใช่ของ driver
2. เขียน adapter ใน `infrastructure/src/services/<ชื่อ>/` implement port นั้น (ไฟล์ละหน้าที่: `get.rs`, `del_prefix.rs` …)
3. middleware ที่ใช้ร่วมกันหลาย service อยู่ใน `infrastructure/src/middleware/`
4. `application/*` **ห้าม** import driver ตรง ๆ

## 6. เช็กลิสต์

- [ ] ส่งข้อความเข้า topic แล้ว worker ทำงาน: `kubectl -n arch exec deploy/arch-streaming -- rpk topic consume <topic> -n 5`
- [ ] ข้อความซ้ำ/ข้อความเสีย ไม่ทำให้ worker ค้างหรือทำซ้ำ (มี test)
- [ ] ปิดด้วย Ctrl+C ได้ภายในไม่กี่วินาที (ลองตอนมี SSE เปิดอยู่)
- [ ] Redpanda ล่ม/restart แล้ว worker ต่อใหม่เอง (log "retrying in 5 s")
- [ ] เว็บเรียกผ่าน Trunk proxy ได้ (`curl localhost:3000/api/v1/<svc>/…` ด้วยคุกกี้ล็อกอิน)
- [ ] ผู้ใช้อื่นเข้าถึงข้อมูลของกันและกันไม่ได้ (เช็กความเป็นเจ้าของ ตอบ 404 ไม่ใช่ 403 เพื่อไม่เผยว่ามีอยู่ — แบบ api-report)
- [ ] `docs/architecture/09_…` และ `docs/programs/…` อัปเดตแล้ว

## 7. กับดักที่เจอมาแล้ว

| อาการ | สาเหตุ / วิธีแก้ |
|---|---|
| Ctrl+C แล้ว service ไม่ยอมหยุด | SSE ที่เปิดค้างทำให้ `with_graceful_shutdown` ไม่จบ → ปิด stream ทั้งหมด (`close_all`) และใช้ watchdog |
| ลิงก์ presigned ของ Garage "Invalid signature" ผ่าน dev proxy | proxy ตาม redirect เองแล้ว Host เปลี่ยน → ให้ API ตอบ **JSON `{url}`** ให้เบราว์เซอร์เปิดเอง ไม่ใช้ redirect |
| topic หายหลัง Redpanda เกิดใหม่ | ปัจจุบันใช้ PVC แล้ว; ถ้าสร้าง cluster ใหม่ต้อง `make topics` |
| ข้อความเก่าถูกประมวลผลย้อนหลังตอนตั้ง group ใหม่ | `RedpandaEventConsumer` ตั้ง `auto.offset.reset=earliest` — group ใหม่อ่านตั้งแต่ต้น topic; handler ต้อง idempotent และถ้าไม่ต้องการ ให้ใช้ชื่อ group เดิมต่อเนื่อง |
| `nest("/")` ของ router ย่อยแล้ว proxy จับไม่ถูก | ใช้ path จริง เช่น `/items` |
