---
id: "019ca1ee-0000-7000-8003-000000000001"
code: "DBPG006"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg006"
icon: "TranslationOutlined"
title_th: "ข้อความแปลภาษา"
title_en: "i18n Labels"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - i18n
  - label
  - screen
---

# 🌐 DBPG006 - ข้อความแปลภาษา (i18n Labels)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG006`
> - **ชื่อภาษาไทย**: ข้อความแปลภาษา
> - **Title (EN)**: i18n Labels
> - **Route Path**: `/db/dbpg006`
> - **Icon**: `TranslationOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Parent Folder**: `DBPG000` (ข้อมูลหลัก / Master Data)
> - **Database UUID**: `019ca1ee-0000-7000-8003-000000000001`
> - **Seed**: [`sql/52_program_catalogue.sql`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/sql/52_program_catalogue.sql)
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับผู้ดูแลระบบจัดการพจนานุกรมข้อความแปลภาษาของหน้าจอ (UI Translation Dictionary) ที่เก็บในตาราง `db_label` เดิมซึ่งอ่านอย่างเดียวผ่าน `GET /api/v1/i18n/...` ผู้ดูแลค้นหา เพิ่ม แก้ไข และลบข้อความได้โดยไม่ต้องแก้ SQL

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)

### 2.1 วัตถุประสงค์ (Objectives)
- ให้ Admin ดูและแก้ข้อความแปลภาษาทุกภาษา (ปัจจุบัน `th` / `en`) ของทุกหมวดในที่เดียว
- 1 **รายการ (entry)** = `(category, key)` หนึ่งคู่ ซึ่งมี 1 แถวใน `db_label` ต่อภาษา

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- **หมวด (`category`)**: 1–100 ตัวอักษร ใช้ได้เฉพาะ `A-Z a-z 0-9 _ . -` (ตัดช่องว่างหัวท้าย) เช่น `menu`, `error`, `supg003`
- **คีย์ (`key`)**: ไม่ว่าง ยาวไม่เกิน 300 ตัวอักษร (ตัดช่องว่างหัวท้าย) เช่น `column.thai_name`
- **ข้อความ (`value`)**: ต่อ 1 ภาษา ยาวไม่เกิน 2000 ตัวอักษร (ตัดช่องว่างหัวท้าย); ช่องที่ว่าง = ไม่มีข้อความสำหรับภาษานั้น
- ฟอร์มแสดงช่องกรอก 1 ช่องต่อภาษาที่เปิดใช้งาน พร้อมชื่อภาษาไทยของภาษานั้น (`db_language_label`)

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- รายการซ้ำ `(category, key)` ไม่ได้ (สร้างซ้ำตอบ 409 `DUPLICATE`)
- บันทึก (สร้าง/แก้ไข) เขียนทุกภาษาใน transaction เดียว: ข้อความไม่ว่าง → เพิ่มแถวใหม่ (UUIDv7, `row_version = 1`) หรือแก้แถวเดิม (`row_version + 1`); ข้อความว่าง → ลบแถวของภาษานั้น
- ต้องมีข้อความอย่างน้อย 1 ภาษา และทุก `language_id` ต้องเป็นภาษาที่เปิดใช้งาน
- หมวดและคีย์แก้ไขภายหลังไม่ได้ (โค้ดอ้างอิงข้อความด้วย `<category>.<key>`) ลบแล้วสร้างใหม่แทน
- ลบรายการ = ลบทุกแถวของ `(category, key)` (ไม่มีตารางอื่นอ้างอิง `db_label`)
- **Cache**: พจนานุกรม `GET /api/v1/i18n/...` อ่านจาก `db_label` ทุกครั้ง ไม่มี cache ฝั่งเซิร์ฟเวอร์ จึงไม่ต้องล้าง cache เมื่อแก้ไข (ไคลเอนต์ที่โหลดพจนานุกรมไปแล้วจะเห็นค่าใหม่เมื่อโหลดหน้าใหม่)

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- API อยู่หลัง Ory Kratos session และตรวจสิทธิ์ด้วย `routers/access.rs::require()` กับโปรแกรม `DBPG006`
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)** (กลุ่ม ADMIN ได้ครบทั้งสาม):
  - `READ`: ดูรายการ ค้นหา และเปิดฟอร์ม (`GET`)
  - `WRITE`: เพิ่ม แก้ไข (`POST` / `PUT`)
  - `APPROVE`: ลบรายการ (`DELETE`)

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_label` (id, language_id, category, key, value, change_by, row_version) — unique `(category, key, language_id)`
- `db_language`, `db_language_label` (รายชื่อภาษาและชื่อที่แสดง)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg006/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg006/) (`Dbpg006Page`, `Dbpg006LabelPage`)
  - `/db/dbpg006` รายการข้อความ (เลือกหมวด, ค้นหาหมวด/คีย์/ข้อความ, แสดงข้อความทุกภาษา, แบ่งหน้า 25 รายการ)
  - `/db/dbpg006/label[?category=&key=]` ฟอร์มข้อความ (ระบุ `category` + `key` เพื่อแก้ไข, `?category=` เลือกหมวดล่วงหน้าตอนสร้าง; ตั้งชื่อหมวดใหม่ได้; ปุ่มลบอยู่ซ้ายของท้ายฟอร์มเมื่อแก้ไข)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg006` ([`dbpg006_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg006_router.rs)):
  - `GET /languages`, `GET /categories`
  - `GET /labels` (`?category=&search=&page=&limit=`, ค่าเริ่มต้น 25 สูงสุด 200)
  - `GET /labels/entry?category=&key=`
  - `POST /labels`, `PUT /labels` (body `{category, key, values: [{language_id, value}]}`)
  - `DELETE /labels?category=&key=` (204)
  - ข้อผิดพลาด: Validation 400, NotFound 404, Duplicate 409, Database 500
- **Application Layer**: [`features/db/dbpg006/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg006/) (`Dbpg006Queries`, `Dbpg006Commands`, `Dbpg006CommandExecutor`)
  - การแบ่งหน้าทำบนคู่ `(category, key)` ที่ไม่ซ้ำ (GROUP BY ใน SQL) แล้วจึงโหลดแถวของคู่ในหน้านั้น; `total` นับเป็นรายการ ไม่ใช่แถว
- **Domain Layer**: `domain::entity::{db_label, db_language, db_language_label}`
- **Tests**: `application/master/tests/dbpg006_test.rs`, `handler/api/master/tests/dbpg006_router_test.rs`
