# ALPG001 — Process Monitor (ติดตามกระบวนการ)

ดูว่า**งานเบื้องหลังที่ส่งผ่าน Redpanda** สำเร็จหรือไม่ และสั่ง **ประมวลผลซ้ำ (reprocess)** ได้
ปัจจุบันมี process 1 ชนิดคือสร้างรายงาน CSV ของ ALPG000 (`report_kind = audit_log_csv`, แถวใน `rp_report_job`)

- เส้นทางเว็บ: `/al/alpg001` (เมนู ตรวจสอบ → ติดตามกระบวนการ) • โปรแกรม `ALPG001` ใน `sql/52_program_catalogue.sql`
- API: `/api/v1/al/alpg001` (api-master) — ดู READ, สั่ง reprocess ต้อง **WRITE** บน `ALPG001`
- โค้ด: `application/master/src/features/al/alpg001`, `handler/api/master/src/routers/alpg001_router.rs`, `handler/web/application/master/src/pages/al/alpg001`

## สถานะของ process

| สถานะ | ความหมาย |
|---|---|
| `queued` | รอ worker รับ (ถูกสั่ง reprocess แล้ว) |
| `running` | worker กำลังสร้าง |
| `succeeded` | สำเร็จ (มีไฟล์ใน Garage) |
| `failed` | ไม่สำเร็จ — เหตุผลอยู่ในคอลัมน์ "ผลลัพธ์" |

`queued` / `running` ที่ไม่ขยับเกิน **15 นาที** ถือว่า **ค้าง** (`stuck`, worker ตายหรือคำขอหาย) — แสดงป้าย "ค้างนานผิดปกติ" และ reprocess ได้

## หน้าจอ

- ตัวกรอง: ช่วงวันที่ขอ (`DateRangePicker`, ค่าเริ่มต้น "ทุกช่วงเวลา"), ปุ่มสถานะพร้อมตัวนับ, ค้นหาผู้ขอ
- คอลัมน์: เวลาที่ขอ, กระบวนการ (+ id), ผู้ขอ, สถานะ, ผลลัพธ์ (จำนวนแถว/ขนาด/ชื่อไฟล์ หรือข้อความ error), ใช้เวลา, จัดการ
- ระหว่างมี process `queued`/`running` ตารางรีเฟรชเองทุก 4 วินาที

## Reprocess

`POST /api/v1/al/alpg001/processes/{id}/reprocess` → 202

1. ตรวจว่าทำได้: `succeeded` / `failed` ได้เสมอ (สั่งซ้ำเพื่อสร้างไฟล์ที่หายจาก Garage ได้), `queued` / `running` เฉพาะที่ค้าง; มิฉะนั้น **409** `PROCESS_NOT_REPROCESSABLE`
2. UPDATE แบบมีเงื่อนไขครั้งเดียว: แถวกลับเป็น `queued`, ล้าง error, `started_at = now` (สองคนกดพร้อมกันได้คิวเดียว)
3. ส่ง `ReportJobMessage` เดิม (จาก `rp_report_job.filter`) ไป topic `report.requests` อีกครั้ง พร้อม `reprocess: true`
   — ถ้า broker ล่ม: คืนแถวกลับเป็นเดิมและตอบ **503** `PROCESS_QUEUE_UNAVAILABLE`
4. api-report รับข้อความ: `reprocess` จะรันเฉพาะแถวที่เป็น `queued`/`running` (ข้อความซ้ำหลังรันเสร็จจึงไม่รันซ้ำ) แล้วสร้างไฟล์ใหม่ทับ `reports/<id>.csv`, อัปเดตแถว และแจ้งผู้ขอตามเดิม

หมายเหตุ: ผู้ขอเดิมได้รับแจ้งเตือน `report_ready` / `report_failed` อีกครั้งเมื่อรันซ้ำเสร็จ

## ขอบเขต / ข้อจำกัด

- เห็นเฉพาะ process ที่ worker **เริ่มทำแล้ว** (แถวถูกสร้างตอนรับงาน); คำขอที่ค้างอยู่ใน topic และยังไม่มี worker รับจะยังไม่ปรากฏ — ดูได้ที่ ALPG000 → Event: คำขอรายงาน
- การประมวลผลของ notification (`notifications` topic) ยังไม่อยู่ในหน้านี้
