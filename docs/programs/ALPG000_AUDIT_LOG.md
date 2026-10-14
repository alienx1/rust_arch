---
id: "019ca1ed-0000-7000-8003-000000000031"
code: "ALPG000"
system_code: "AL"
module_code: "ALMT"
route_path: "/al/alpg000"
icon: "FileSearchOutlined"
title_th: "บันทึกการตรวจสอบ"
title_en: "Audit Log"
parent_group: "ALPG000_GROUP"
status: "Implemented (Quickwit; no Redpanda / S3 archive yet)"
tags:
  - program
  - audit
  - compliance
  - quickwit
  - screen
---

# 🔍 ALPG000 - บันทึกการตรวจสอบ (Audit Log)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `ALPG000`
> - **ชื่อภาษาไทย**: บันทึกการตรวจสอบ
> - **Title (EN)**: Audit Log
> - **Route Path**: `/al/alpg000`
> - **Icon**: `FileSearchOutlined` (`icondata_ai`)
> - **System**: `AL` (ตรวจสอบ / Audit)
> - **Module**: `ALMT` (ตรวจสอบระบบ / Audit Maintenance)
> - **Parent Folder**: `ALPG000_GROUP` (ตรวจสอบ / Audit)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000031`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับตรวจสอบและสืบค้นประวัติกิจกรรมที่เกิดขึ้นทั้งหมดในระบบ (Security & Business Audit Trail) เพื่อการปฏิบัติตามกฎหมาย (Compliance), การสืบสวนข้อผิดพลาด และความโปร่งใส ดึงข้อมูลการค้นหาความเร็วสูงผ่าน **Quickwit OpenSearch-compatible Engine** บน Object Storage (Garage S3)

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น ให้ผู้ตรวจสอบ (Auditor) สืบค้นเหตุการณ์ย้อนหลัง, ดู Before/After JSON diff]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **เกณฑ์การค้นหาและตัวกรอง (Search Filters)**:
  - ช่วงวันและเวลา (Timestamp Range): From - To
  - ผู้กระทำ (Actor / User): `user_id`, `username`, `email`
  - หมายเลข IP และ User Agent
  - โปรแกรมเป้าหมาย (Target Program): เช่น `DBPG001`, `SUPG004`
  - การกระทำ (Action): `CREATE`, `UPDATE`, `DELETE`, `LOGIN`, `APPROVE`
  - ค้นหาคำเต็ม (Full-text Search): ค้นหาใน Payload/Diff
- [ ] **การแสดงผลรายละเอียดบันทึก (Event Details)**:
  - Event ID & Timestamp (UTC / Local ICT)
  - Correlation ID / Trace ID
  - ตารางและ Record ID ที่ได้รับผลกระทบ
  - ข้อมูลก่อนหน้าและหลังแก้ไข (Before / After Snapshot Diff)

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Immutability**: บันทึก Audit Log ทั้งหมดต้องเป็นแบบ Append-only / Write Once Read Many (WORM) ห้ามแก้ไขหรือลบผ่านหน้าจอนี้โดยเด็ดขาด
- [x] **Export CSV**: ปุ่ม "ส่งออก CSV" ในแถบตัวกรอง → `POST /api/v1/al/alpg000/exports` (พารามิเตอร์ตัวกรองเดียวกับรายการ ไม่มี `page`/`limit`) ต้องมีสิทธิ์ READ ของ ALPG000 ทำงานเบื้องหลังผ่าน Redpanda ไม่ค้างหน้าจอ:
  1. api-master ตรวจสิทธิ์/ตัวกรอง แล้ว publish `ReportJobMessage` ลง topic `report.requests` (key = job id) → ตอบ **202** `{job_id}` (broker ล่ม → 503)
  2. `api-report` (worker, consumer group `report-worker`) อ่านข้อความ → ดึง event ทั้งหมดที่ตรงตัวกรองจาก Quickwit ด้วย scroll ทีละ 5,000 แถว → เขียนเป็น CSV ขึ้น Object Storage (Garage) แบบ multipart ทีละ 8 MiB (ไม่เก็บทั้งไฟล์ในหน่วยความจำ) ที่ `reports/<job id>.csv` เพดาน 5,000,000 แถว
  3. สถานะงานอยู่ในตาราง `rp_report_job` (running → succeeded | failed) และแจ้งผู้ขอผ่าน `NotificationService` (adapter = publish ลง topic `notifications`) → **notification service** (NTPG000) เก็บลง `nt_notification` แล้ว push ผ่าน SSE
  4. เว็บรับ noti ที่กระดิ่งใน topbar (ทุกหน้า ทุกแท็บ ทุกอุปกรณ์ของผู้ใช้นั้น) → "รายงานของฉัน" (RPPG000, `/rp/rppg000`) → ดูตัวอย่าง (100 แถวแรก อ่านด้วย ranged GET) หรือดาวน์โหลด (api-report ตอบ signed URL อายุ 5 นาทีเป็น JSON แล้วเบราว์เซอร์เปิดเอง ไม่ใช้ redirect เพราะ proxy ที่ตาม redirect เองทำให้ลายเซ็น Host ผิด)
  - ผู้ขอเท่านั้นที่เห็นไฟล์ของตัวเอง (ของคนอื่น = 404)
  - ไฟล์: UTF-8 + BOM (Excel อ่านภาษาไทยถูก), CRLF, 1 event ต่อ 1 แถว, ชื่อ `audit-log-YYYYMMDD-HHMMSS.csv`
  - คอลัมน์: `time_th` (UTC+7) `time_utc` `kind` `user_id` `username` `email` `ip` `method` `path` `query` `status` `duration_ms` `program` `action` `page` `element` `label` `tables` `summary` `changes` `request_id` — ไม่รวม header/body (ใหญ่เกินอ่าน); `changes` ของ data_change เป็นบรรทัดเดียว `table OP row_id: col: เดิม -> ใหม่` คั่นแต่ละแถวด้วย ` | ` (ตัดที่ 1,500 ตัวอักษร)
  - ค่าที่ขึ้นต้นด้วย `= + - @` ถูกใส่ `'` นำหน้า กันสูตรใน Excel (CSV injection)
  - Store ล่มกลางทาง → งาน `failed` (abort multipart, ไม่ทิ้งไฟล์ครึ่งเดียว) + noti แจ้งล้มเหลว
  - ข้อจำกัด: Excel เปิดได้ไม่เกิน 1,048,576 แถวต่อชีต — ไฟล์ใหญ่กว่านี้ให้แบ่งด้วยตัวกรองช่วงเวลา หรือเปิดด้วย Power Query / DB tool; ผู้ใช้ที่ออฟไลน์ตอนไฟล์เสร็จไม่ได้ noti แต่เห็นงานใน RPPG000; job row เกิดเมื่อ worker รับงาน (ถ้า worker ล่ม งานรออยู่ใน Redpanda)
- [ ] **Export PDF**: ยังไม่ทำ

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ค้นหาและดูบันทึก Audit Log
  - `WRITE`: *(ปิดการใช้งาน - ไม่มีฟังก์ชัน Write ในระบบ Audit Log)*
  - `APPROVE`: อนุมัติการ Export ข้อมูลประวัติการตรวจสอบขนาดใหญ่
- **Access Restriction**: สงวนสิทธิ์สำหรับกลุ่ม Auditor, Security Officer และ Super Admin

---

## 🗄️ 4. แหล่งจัดเก็บข้อมูล (Storage Engine)
- **Search Engine**: Quickwit 0.8 (Index: `audit-logs-v1`, mapping ที่ `infrastructure/src/services/quickwit/audit_logs_v1_index.json`) แอปสร้าง index เองเมื่อยังไม่มี (รวมถึงหลัง Quickwit รีสตาร์ท)
- **การบันทึก**: ส่งตรงเข้า Quickwit ผ่าน REST ingest เป็นชุด (ทุก 1 วินาที หรือครบ 200 รายการ) ผ่านคิวที่ไม่ถ่วง request; Quickwit ล่ม = ทิ้งเหตุการณ์ (มี retry สั้น ๆ) ไม่กระทบการใช้งาน
- **ยังไม่ได้ทำ**: Redpanda topic `audit.events`, เก็บถาวร Garage S3 `arch-audit`, Export CSV/PDF, บันทึก SELECT (อ่านข้อมูล)
- **Data change (`kind=data_change`)**: trigger `al_capture_change` (ใน `sql/01_ysql.sql`) บนทุกตารางจับ INSERT / UPDATE / DELETE (UPDATE ที่เปลี่ยนแค่ `row_version` ไม่ถูกจับ) พร้อมแถวก่อน/หลัง (ปิดบังฟิลด์ password/secret/token) ลงตาราง outbox `al_data_change` ใน transaction เดียวกัน แล้ว `spawn_data_change_shipper` รวมแถวของ API request เดียวกัน (`request_id`) เป็น **1 event** (`changes` = รายการแถวที่เปลี่ยน: table, op, row_id, columns, old, new; `detail` = สรุปรายตาราง) ส่งเข้า Quickwit ทุก 2 วินาทีและลบออกจาก outbox ผู้กระทำมาจาก session setting `app.actor_id` / `app.actor_name` / `app.request_id` ที่ middleware `audit_trail` ตั้งให้ทุก connection ของ request เขียน (`infrastructure::with_actor` + `track_actor`); `request_id` = `event_id` ของ API event เดียวกัน การเขียนที่ไม่มีผู้กระทำ (seed, migration, worker) จะไม่ถูกจับ
- **หมายเหตุ (dev)**: ข้อมูลของ Quickwit อยู่บน PVC `arch-logging-data` (local-path) — pod รีสตาร์ทข้อมูลไม่หาย แต่ลบ PVC/cluster แล้วหาย
- **ตั้งค่า**: `QUICKWIT_URL` (ค่าเริ่มต้น `http://localhost:7280`), `AUDIT_LOG_ENABLED` (ค่าเริ่มต้น true)

### 4.1 เหตุการณ์ที่บันทึก (`AuditEvent`, `application/core/src/traits/audit_log.rs`)
| kind | มาจาก | เนื้อหา |
|---|---|---|
| `api` | เซิร์ฟเวอร์ (system log) ทุก request ของ `/api/v1/**` | ใคร (`user_id` = Kratos identity id, username, email, session, IP, user agent, ภาษา), เรียกอะไร (method, path, query, โปรแกรม, action), status, ระยะเวลา, headers, request/response body |
| `page_view` | เว็บ | เปิดหน้าไหน |
| `click` | เว็บ | กดปุ่ม/ลิงก์อะไร (องค์ประกอบ + ข้อความบนปุ่ม + หน้า) ไม่เก็บค่าที่พิมพ์ในช่องกรอก |
| `action` | เว็บ | process ที่ทำสำเร็จ เช่น `DBPG004.save`, `SUPG003.matrix.save`, `SUPG004.recovery_link` |

- **การปิดบังข้อมูล**: headers `cookie`/`authorization`/`x-api-key` ฯลฯ และ field ใน body/query ที่ชื่อมี password/secret/token/cookie/api key/credential ถูกแทนที่ด้วย `***`
- **Body**: แปลง CBOR เป็น JSON (UUID แสดงเป็นรูป hyphenated) เก็บไม่เกิน 16 KiB; body ที่ไม่ทราบขนาดล่วงหน้า/เกิน 256 KiB ไม่เก็บ (ระบุขนาดแทน)
- **ข้ามการบันทึก**: `/api/v1/audit/**`, การอ่านของหน้า ALPG000 เอง, พจนานุกรม i18n, OPTIONS

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/al/alpg000/` — `/al/alpg000` (ตารางพร้อมตัวกรอง: ช่วงเวลา, ชนิด, สถานะ 2xx–5xx, ผู้ใช้, method, โปรแกรม, path, ค้นข้อความ) และ `/al/alpg000/log/{id}` (รายละเอียด + request/response JSON); ตัวจับกิจกรรมที่ `src/audit/` (เปิดหน้า, คลิก, `track_action`) ส่งเป็นชุดทุก ~3 วินาที
- **Backend API** (`/api/v1/al/alpg000`, ต้องมีสิทธิ์ READ): `GET /logs`, `GET /logs/{id}`, `GET /status`; ผู้ใช้ทุกคนส่งกิจกรรมของตัวเองได้ที่ `POST /api/v1/audit/events` (ใช้ session อย่างเดียว) ถ้า Quickwit ไม่พร้อมตอบ 503 `AUDIT_UNAVAILABLE`
- **Application Layer**: `application/master/src/features/al/alpg000/`; port `AuditLogPort` ที่ `application/core`
- **Infrastructure Layer**: `infrastructure/src/services/quickwit/` (`QuickwitAuditLog`: คิว + flusher + query builder); middleware `infrastructure/src/middleware/audit/audit_trail.rs`
- **ข้อจำกัดการค้นหา**: `path` ค้นแบบ token/prefix (เช่น `dbpg004`) ไม่ใช่ substring ตัวอักษรใดก็ได้; ค้นข้อความถูกตัดคำตาม tokenizer ของ Quickwit
- **Tests**: `alpg000_test.rs`, `alpg000_router_test.rs`, `audit_trail_test.rs`, `infrastructure/tests/quickwit_audit_query_test.rs` (มี integration test ที่ยิง Quickwit จริงแบบ `--ignored`)

## Event จาก Redpanda (`event_report_request`, `event_notification`)

ตัวกรอง "ประเภท" มีสอง kind เพิ่ม ซึ่งอ่านจาก index `events-report-requests-v1` / `events-notifications-v1` (Kafka source ของ Quickwit, ตั้งด้วย `make event-indexes` — ดู `NTPG000_NOTIFICATIONS.md`) แทน `audit-logs-v1`:

- เลือกได้ครั้งละ 1 kind (ไม่รวมกับ kind อื่น); ใช้ได้เฉพาะตัวกรองเวลา, ผู้ใช้ (ผู้ขอ / ผู้รับ) และค้นข้อความ — ตัวกรอง method / status / program / path ใช้กับ event เหล่านี้ไม่ได้
- ปุ่มดูรายละเอียดทำงานตามปกติ (`GET /logs/{id}` ค้น job id / notification id)
- ส่งออก CSV ไม่รองรับ (คำขอ export ที่เลือก kind นี้ตอบ 400)
- โค้ด: `infrastructure/src/services/quickwit/event_stream_query.rs`, ทดสอบที่ `infrastructure/tests/event_stream_query_test.rs`

## ช่วงวันที่ (date range)

ตัวกรองเวลาเป็นปุ่ม "ช่วงวันที่" (คอมโพเนนต์ร่วม `web_package::ui::component::DateRangePicker`) แทนปุ่มช่วงเวลาด่วนเดิม กดแล้วเปิด popover:

- ซ้าย: preset ด่วน — วันนี้, เมื่อวาน, 7 วันล่าสุด, 30 วันล่าสุด, เดือนนี้, เดือนที่แล้ว, ทุกช่วงเวลา
- ขวา: ปฏิทินรายเดือน (ปีแสดงเป็น พ.ศ. ในภาษาไทย) กดวันแรก เลื่อนเมาส์เพื่อดูแถบช่วงที่จะเลือก แล้วกดวันสุดท้ายเพื่อใช้ทันที; กดนอก popover หรือ Esc เพื่อปิด; บนมือถือ popover กว้างเต็มจอ
- ค่าเริ่มต้น = **วันนี้**; "ทุกช่วงเวลา" = ไม่จำกัดเวลา
- วันที่ตีความเป็นเวลาท้องถิ่นของ browser: `from` = 00:00 ของวันแรก, `to` = 00:00 ของ **วันถัดจากวันสุดท้าย** (exclusive) จึงรวมทั้งวันสุดท้าย — เช่น 12 ต.ค. ใน UTC+7 ส่ง `from=2026-10-11T17:00:00Z&to=2026-10-12T17:00:00Z`
- "ส่งออก CSV" ใช้ช่วงวันที่เดียวกัน
