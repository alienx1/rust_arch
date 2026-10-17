# 🔔 NTPG000 - การแจ้งเตือน (Notifications)

บริการกลางที่ทุก service ใช้บอกผู้ใช้ว่าเกิดอะไรขึ้น (รายงานเสร็จ, งานรออนุมัติ ฯลฯ) แสดงที่ไอคอนกระดิ่งใน topbar ทุกหน้า และหน้ารายการ `/nt/ntpg000`

## Flow

```text
service ใดก็ได้ ──NotificationService.notify()──▶ Redpanda topic `notifications` (key = recipient)
                                                      │
                                                      ▼
                                      api-notification (consumer group `notification-service`)
                                       1. เก็บลง nt_notification (id = UUIDv7 ของ noti → ส่งซ้ำไม่เกิดแถวซ้ำ)
                                       2. push ผ่าน SSE ไปทุก session ของ recipient
                                                      │
                                                      ▼
                                web: กระดิ่ง (จำนวนที่ยังไม่อ่าน + 10 รายการล่าสุด) / หน้า NTPG000
```

- **ผู้ส่ง** ใช้ port `NotificationService` (`application-core`) adapter `RedpandaNotificationPublisher` (`infrastructure`) ไม่ต้องรู้จัก SSE หรือฐานข้อมูล: `notify(username, Notification { id, kind, title, body, data, link, created_at })`
- **ผู้รับ** ระบุด้วย Kratos username (ทุกแท็บ/อุปกรณ์ที่ล็อกอินด้วยผู้ใช้นั้นได้พร้อมกัน; ผู้ใช้อื่นไม่ได้)
- `link` = path ในเว็บที่เปิดเมื่อกดแถว (ต้องขึ้นต้นด้วย `/`; ลิงก์ออกนอกเว็บถูกทิ้ง กัน phishing)
- สถานะ "อ่านแล้ว" อยู่ที่ server (`read_at`) และมี event `changed` ทาง SSE ให้ทุกแท็บโหลดใหม่ → กดอ่านแท็บหนึ่ง แท็บอื่นเปลี่ยนตามทันที
- **หลายภาษา**: ผู้ส่งเก็บข้อความของตัวเอง (`title`/`body`) เป็นค่าสำรอง และใส่ค่าที่ใช้ประกอบข้อความไว้ใน `data` (เช่น `report_ready` → `{job_id, file_name, rows}`, `report_failed` → `{job_id, reason}`) เว็บหา `ntpg000.kind.<kind>.title` / `.body` ในพจนานุกรมของภาษาที่เลือก (`db_label`, seed `sql/83_i18n_ntpg000_kinds_sound.sql`) แล้วแทนที่ `{ชื่อ}` ด้วยค่าใน `data` — สลับภาษาแล้วข้อความเปลี่ยนทันที; kind ที่ไม่มีในพจนานุกรม หรือแจ้งเตือนเก่าที่ `data` ไม่ครบ จะแสดงข้อความที่ผู้ส่งเก็บไว้ **kind ใหม่**: เพิ่ม key สองตัวนี้ (th/en) พร้อมใส่ค่าใน `data`
- **เสียงแจ้งเตือน**: มีแจ้งเตือนใหม่ถึงผ่าน SSE (event `notification` เท่านั้น ไม่ดังตอน `changed`/โหลดหน้า) เว็บเล่นเสียงสองโน้ตสั้นๆ ที่สร้างด้วย Web Audio (ไม่มีไฟล์เสียง) เปิดไว้เป็นค่าเริ่มต้น ปิด/เปิดได้ที่ไอคอนลำโพงในหัวกระดิ่ง (จำไว้ใน `localStorage` คีย์ `notification_sound`) เบราว์เซอร์ไม่ให้เล่นเสียงก่อนผู้ใช้โต้ตอบกับหน้าอย่างน้อยหนึ่งครั้ง (คลิก/กดแป้น) แจ้งเตือนที่มาก่อนหน้านั้นจึงเงียบ (ตัวเลขบนกระดิ่งยังขึ้น); หลายแท็บที่เปิดพร้อมกันจะดังแท็บละครั้ง
- **เพดาน 300 รายการต่อผู้ใช้**: ทุกครั้งที่รับ noti ใหม่ ระบบลบรายการเก่าสุดของผู้ใช้คนนั้นที่เกิน 300 (`MAX_NOTIFICATIONS_PER_USER`, ไม่สนใจว่าอ่านแล้วหรือยัง) — ผู้ใช้อื่นไม่ถูกกระทบ
- คนที่ปิดเว็บอยู่ตอนส่ง จะเห็นเมื่อกลับมา (โหลดรายการจาก server ตอนต่อ SSE)
- **การอ่าน**: แจ้งเตือนถูกทำเป็น "อ่านแล้ว" เมื่อ**กดที่แถวนั้น**เท่านั้น (ในกระดิ่งหรือหน้า NTPG000) หรือกด "อ่านทั้งหมดแล้ว" — เปิด/ปิดกระดิ่งหรือเข้าหน้ารายการเฉยๆ ไม่ถือว่าอ่าน

## API (`api-notification`, พอร์ต 8002, ต้องมี Kratos session)

| Method | Path | ทำอะไร |
|---|---|---|
| GET | `/api/v1/notifications/items?limit=` | รายการของฉัน (ใหม่สุดก่อน, ค่าเริ่มต้น 20, สูงสุด 300) + `unread` ทั้งหมด |
| POST | `/api/v1/notifications/{id}/read` | อ่านแล้ว 1 รายการ (ของคนอื่น/ไม่มี = 404) |
| POST | `/api/v1/notifications/read-all` | อ่านแล้วทั้งหมด |
| GET | `/api/v1/notifications/stream` | SSE: `ready`, `notification` (รายการใหม่), `changed` (สถานะอ่านเปลี่ยน → โหลดใหม่) |

## ข้อมูลและโครงสร้างโค้ด

- ตาราง `nt_notification` (`sql/01_ysql.sql`), entity `domain/src/entity/nt_notification.rs`
- `application/notification` — feature `ntpg000` (ingest / list / mark read) พร้อมเทสต์ใน `application/notification/tests/`
- `handler/api/notification` — consumer + REST + SSE (`make run-notification`)
- เว็บ: `handler/web/application/master/src/pages/nt/ntpg000/` (กระดิ่งอยู่ใน `MasterLayout` ผ่าน `topbar_actions` ของ `MainLayout`)
- Trunk proxy `/api/v1/notifications/ → 8002` (ต้องอยู่ก่อน `/api/v1/`)

## การรันหลาย instance

SSE hub อยู่ใน memory ของแต่ละ instance และทุก instance ต้องเห็นทุกข้อความเพื่อ push ให้ผู้ใช้ที่ต่อกับตัวเอง จึงต้องกำหนด `NOTIFICATION_CONSUMER_GROUP` **ไม่ซ้ำกัน** ต่อ instance (การเก็บลง DB ปลอดภัยแม้ทุกตัวเก็บซ้ำ เพราะ insert ด้วย id เดียวกันจะถูกข้าม)

## เพิ่ม noti ชนิดใหม่

ตั้ง `kind` ใหม่ (ค่าคงที่ใน `notification_kind`), `title`/`body` (ข้อความที่ผู้ใช้เห็น), `data` สำหรับข้อมูลเฉพาะชนิด และ `link` ไปหน้าที่เกี่ยวข้อง แล้วเรียก `notify` ไม่ต้องแก้ notification service หรือหน้าเว็บ

## ข้อจำกัด

- ผู้ส่งแบบ fire-and-forget: ถ้า broker ล่ม noti ไม่ถูกส่ง (บันทึก log) งานต้นทางไม่ล้ม
- ยังไม่มีการลบ/เก็บถาวรเกินกำหนด (ตาราง `nt_notification` โตเรื่อยๆ)
- topic ต้องมีอยู่ก่อน: `make topics`

## เก็บ event ใน Quickwit

ทุกข้อความใน Redpanda topic `report.requests` และ `notifications` ถูกเก็บลง Quickwit ด้วย Kafka source ของ Quickwit เอง (ไม่มีโค้ดเพิ่ม, consumer group `quickwit-events-*`, เก็บ 90 วัน):

| Index | Topic | Timestamp |
|---|---|---|
| `events-report-requests-v1` | `report.requests` | `requested_at` |
| `events-notifications-v1` | `notifications` | `created_at` |

- สร้าง index/source: `make event-indexes` (idempotent, `make topics` เรียกให้อัตโนมัติ) — config อยู่ที่ `kube/arch/applications/arch_logging/indexes/`
- ค้นหา: `curl 'localhost:7280/api/v1/events-notifications-v1/search?query=recipient:<user_id>'`
- Quickwit อยู่บน PVC แล้ว จึงไม่หายเมื่อ pod รีสตาร์ท
