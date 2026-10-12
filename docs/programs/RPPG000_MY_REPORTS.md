---
id: "019ca1ed-0000-7000-8003-000000000040"
code: "RPPG000"
system_code: "AL"
module_code: "ALMT"
route_path: "/rp/rppg000"
icon: "FileDoneOutlined"
title_th: "รายงานของฉัน"
title_en: "My Reports"
parent_group: "ALPG000_GROUP"
status: "Implemented (api-report + S3 + notifications)"
tags:
  - program
  - report
  - async-job
  - storage
  - screen
---

# 📑 RPPG000 — รายงานของฉัน (My Reports)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `RPPG000`
> - **ชื่อภาษาไทย**: รายงานของฉัน
> - **Title (EN)**: My Reports
> - **Route Path**: `/rp/rppg000`
> - **Icon**: `FileDoneOutlined` (`icondata_ai`)
> - **System**: `AL` (ตรวจสอบ / Audit)
> - **Module**: `ALMT` (ตรวจสอบระบบ / Audit Maintenance)
> - **Parent Folder**: `ALPG000_GROUP` (ตรวจสอบ / Audit)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000040`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

ศูนย์รวมไฟล์รายงานและประวัติงานประมวลผลเบื้องหลัง (Background Export / Report Generation) ของผู้ใช้งานที่ล็อกอินอยู่ เมื่อผู้ใช้ส่งออกข้อมูลจากหน้าจอต่างๆ (เช่น ส่งออก Audit Log จาก [[ALPG000_AUDIT_LOG|ALPG000]]) ระบบจะส่งคำขอผ่าน Redpanda ให้ worker สร้างไฟล์และอัปโหลดไปยัง Garage S3 โดยอัตโนมัติ เมื่อเสร็จแล้วระบบจะแจ้งเตือนผ่าน [[NTPG000_NOTIFICATIONS|NTPG000 (Notifications)]] และนำผู้ใช้มาดูตัวอย่างหรือดาวน์โหลดไฟล์ที่หน้านี้

---

## 🔄 ภาพรวมวงจรการทำงาน (End-to-End Architecture Flow)

```text
1. ผู้ใช้กดส่งออก CSV ที่ ALPG000 ────────▶ api-master ส่ง ReportJobMessage เข้า Redpanda topic `report.requests`
                                                                    │
                                                                    ▼
                                           api-report worker (consumer group `report-worker`)
                                            1. บันทึก/อัปเดตสถานะใน rp_report_job ('queued' -> 'running')
                                            2. สตรีมข้อมูลจาก Quickwit / DB ลง CSV (มี UTF-8 BOM)
                                            3. อัปโหลดแบบ CsvObjectWriter ไปยัง Garage S3 (`reports/{id}.csv`)
                                            4. อัปเดตสถานะเป็น 'succeeded' พร้อม row_count, size_bytes
                                            5. ส่งข้อความเข้า topic `notifications`
                                                                    │
                                                                    ▼
                                           api-notification ดัน SSE ไปยังเว็บผู้ใช้
                                                                    │
                                                                    ▼
                                           ผู้ใช้กดข้อความแจ้งเตือน หรือเปิดหน้า /rp/rppg000
                                            • ดูรายการสถานะรายงานทั้งหมด
                                            • ดูตัวอย่างข้อมูล 20 แถวแรกใน Dialog
                                            • ขอ Pre-signed URL ดาวน์โหลดไฟล์จริง
```

---

## 🗄️ โครงสร้างฐานข้อมูลและการจัดเก็บไฟล์

### 1. ตาราง `rp_report_job` (`sql/01_ysql.sql`)
เก็บประวัติคำขอและผลลัพธ์ของแต่ละงาน:
- `id` (UUIDv7, PK): รหัสเฉพาะของงาน
- `report_kind` (varchar(50)): ชนิดรายงาน เช่น `audit_log_csv`
- `requested_by` (varchar(255)): Kratos username ของผู้ขอ
- `filter` (jsonb): เงื่อนไขที่ใช้กรองข้อมูลตอนขอรายงาน
- `status` (varchar(20)): สถานะของงาน (`queued`, `running`, `succeeded`, `failed`)
- `row_count` (bigint): จำนวนแถวข้อมูลทั้งหมดในไฟล์
- `size_bytes` (bigint): ขนาดไฟล์เป็นไบต์
- `file_name` (varchar(255)): ชื่อไฟล์ดาวน์โหลด (เช่น `audit-log-20261012-152000.csv`)
- `object_key` (varchar(500)): path ใน Garage S3 (เช่น `reports/{job_id}.csv`)
- `error` (text): ข้อความ error กรณีล้มเหลว
- `requested_at` / `started_at` / `finished_at` (timestamptz)

### 2. ที่เก็บไฟล์ (Garage S3 Object Storage)
- Bucket: `rust-arch-bucket`
- คีย์: `reports/{job_id}.csv`
- สตรีมผ่าน `CsvObjectWriter` แบ่งพาร์ตละ 5MB เพื่อประหยัดหน่วยความจำเมื่อข้อมูลมีขนาดใหญ่

---

## 🔌 API Endpoints (`api-report`, พอร์ต 8001)

เส้นทางหลักขึ้นต้นด้วย `/api/v1/reports` (ต้องมี Kratos session และเห็นเฉพาะงานของตนเอง):

| Method | Path | ความหมาย | รายละเอียด |
|---|---|---|---|
| GET | `/api/v1/reports/jobs` | รายการรายงานของฉัน | เรียงลำดับจากใหม่สุดไปเก่าสุด รองรับ wire format JSON และ CBOR |
| GET | `/api/v1/reports/jobs/{id}` | ข้อมูลสถานะของงานระบุตาม ID | คืนข้อมูลแถวงาน `ReportJobDto` |
| GET | `/api/v1/reports/jobs/{id}/preview?rows=20` | ดูตัวอย่างแถวเริ่มต้น | สตรีมส่วนหัวและแถวแรกจาก S3 มา parse เป็นตารางตัวอย่าง (ค่าเริ่มต้น 20 แถว) |
| GET | `/api/v1/reports/jobs/{id}/download` | ขอลิงก์ดาวน์โหลดไฟล์ | คืนค่า JSON `{ "url": "..." }` เป็น Pre-signed S3 URL อายุสั้นเพื่อความปลอดภัย |

---

## 💻 หน้าจอผู้ใช้งาน (User Interface & Clean UI)

- **Route Path**: `/rp/rppg000`
- **Component Shell**: ประกอบด้วย Header แนะนำการใช้งาน, ปุ่ม Refresh, และตารางรายการ
- **ตารางแสดงผล (`ReportJobTable`)**:
  - **สร้างเมื่อ**: วันที่และเวลาตาม Timezone ประเทศไทย (UTC+7)
  - **ชนิดรายงาน**: ชนิดและไอคอนประเภทรายงาน
  - **สถานะ**:
    - ⏳ `queued` (รอคิว)
    - 🔄 `running` (กำลังประมวลผล)
    - ✅ `succeeded` (สร้างสำเร็จ)
    - ❌ `failed` (ล้มเหลว พร้อม tooltips แสดงสาเหตุ)
  - **จำนวนแถว / ขนาด**: แสดงจำนวนแถวและฟอร์แมตขนาดไฟล์ (KB, MB)
  - **การจัดการ**:
    - ปุ่ม **"ดูตัวอย่าง"** (เปิด `ReportPreviewDialog` ดูคอลัมน์และข้อมูล 20 แถวแรก)
    - ปุ่ม **"ดาวน์โหลด"** (เปิดดาวน์โหลดผ่าน Pre-signed S3 URL)
- **การเชื่อมโยงกับ Process Monitor**: กรณีเกิดข้อผิดพลาดหรือกระบวนการค้าง แอดมินสามารถตรวจสอบและสั่งรันซ้ำได้ที่ [[ALPG001_PROCESS_MONITOR|ALPG001 (Process Monitor)]]

---

## 🧱 โครงสร้างโค้ดในระบบ (Code Layout)

- **Domain Entity**: `domain/src/entity/rp_report_job.rs`
- **Application Layer**: `application/report/src/features/rppg000/`
  - `commands/`: `list_jobs.rs`, `get_job.rs`, `download_job.rs`, `preview_job.rs`, `run_report_job.rs`, `csv_object_writer.rs`, `audit_csv_rows.rs`
  - `queries/`: `db_rppg000_queries.rs`
  - `models/`: DTOs, ค่าคงที่ `job_status`, `DOWNLOAD_LINK_SECS`, `UPLOAD_PART_BYTES`
- **API Delivery**: `handler/api/report/src/routers/rppg000_router.rs`
- **Frontend SPA**: `handler/web/application/master/src/pages/rp/rppg000/`
  - `components/`: `report_job_table.rs`, `report_preview_dialog.rs`
  - `providers/`: `rppg000_context.rs`
  - `services/`: `rppg000_service.rs`
  - `screens/`: `rppg000_screen.rs`
- **Unit & Integration Tests**: `application/report/tests/rppg000_test.rs`

---

## 🔗 ความเชื่อมโยงที่เกี่ยวข้อง (Cross-References)
- [[ALPG000_AUDIT_LOG|ALPG000 — บันทึกการตรวจสอบ]]: ต้นทางของคำขอส่งออกรายงาน CSV
- [[ALPG001_PROCESS_MONITOR|ALPG001 — ติดตามกระบวนการ]]: หน้าสำหรับมอนิเตอร์และสั่ง Reprocess งาน
- [[NTPG000_NOTIFICATIONS|NTPG000 — การแจ้งเตือน]]: ระบบ Push แจ้งเตือนเมื่อรายงานเสร็จสมบูรณ์
- [[07_EVENT_STREAMING_OBSERVABILITY|Chapter 7: Event Streaming & Observability]]: สถาปัตยกรรม Redpanda `report.requests` และ Quickwit Index
