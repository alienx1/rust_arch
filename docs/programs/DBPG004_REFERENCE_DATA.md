---
id: "019ca1ed-0000-7000-8003-000000000014"
code: "DBPG004"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg004"
icon: "GroupOutlined"
title_th: "ข้อมูลอ้างอิงระบบ"
title_en: "Reference Data"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - reference-data
  - lookup
  - screen
---

# 📚 DBPG004 - ข้อมูลอ้างอิงระบบ (Reference Data / Master Lookup)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG004`
> - **ชื่อภาษาไทย**: ข้อมูลอ้างอิงระบบ
> - **Title (EN)**: Reference Data
> - **Route Path**: `/db/dbpg004`
> - **Icon**: `GroupOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Parent Folder**: `DBPG000` (ข้อมูลหลัก / Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000014`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการตารางค่าอ้างอิงกลาง (Generic Master Lookup Table) ของทั้งระบบ เช่น สถานะเอกสาร, ประเภทการชำระเงิน, หน่วยนับ, สกุลเงิน (`db_currency`), และภาษาของระบบ (`db_language`) เพื่อป้องกันการ Hardcode ค่าคงที่ใน Source Code

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น ให้ Admin เพิ่มหมวดหมู่ Lookup และตัวเลือกใน Dropdown ต่างๆ]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **หมวดหมู่ข้อมูล (Lookup Category)**:
  - รหัสหมวดหมู่ (Category Code): เช่น `gender`, `title_name`, `payment_method`
  - ชื่อหมวดหมู่ภาษาไทย / อังกฤษ
- [ ] **รายการตัวเลือก (Lookup Items)**:
  - รหัสตัวเลือก (Item Code): เช่น `MALE`, `FEMALE`
  - ป้ายชื่อแสดงผล (Labels): รองรับหลายภาษาตาม `db_language`
  - ลำดับการแสดงผล (Sort Order): ตัวเลขจัดเรียงใน UI Dropdown
  - ค่าเริ่มต้น (Is Default): boolean
  - สถานะเปิดใช้งาน: `active`
- [ ] **สกุลเงินและอัตราแลกเปลี่ยน (Currency Reference)**:
  - รหัสสกุลเงิน (ISO 4217 เช่น `THB`, `USD`, `EUR`)
  - สัญลักษณ์สกุลเงิน (Symbol เช่น `฿`, `$`)
  - ทศนิยม (Decimals: 2)

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Cache Invalidation**: เมื่อมีการแก้ไข Master Lookup ระบบต้องส่ง Event ไปล้าง Cache ใน Valkey ทันที
- [ ] **System Protected Rows**: รายการที่เป็น System Core Lookup (เช่น สิทธิ์ `READ`, `WRITE`) ต้องล็อกไม่ให้ลบ

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- API อยู่หลัง Ory Kratos session และตรวจสิทธิ์ด้วย `routers/access.rs::require()` กับโปรแกรม `DBPG004`: `READ` = ดู/ค้นหา (`GET`), `WRITE` = เพิ่ม/แก้ไข (`POST`/`PUT`), `APPROVE` = ลบ (`DELETE`)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ดูและค้นหาค่า Lookup (`GET` ทุกเส้นทาง)
  - `WRITE`: เพิ่มหรือแก้ไขค่า Lookup (`POST` / `PUT`)
  - `APPROVE`: ลบค่า Lookup (`DELETE`)

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_master_lookup` (id, category, code, sort_order, active, ...)
- `db_master_lookup_label` (master_lookup_id, language_id, label)
- `db_currency`, `db_currency_label`
- `db_language`, `db_language_label`

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg004/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg004/) (`Dbpg004Page`, `Dbpg004LookupPage`)
  - `/db/dbpg004` รายการค่าอ้างอิง (กรองหมวดหมู่ที่มีจำนวนรายการ, ค้นหารหัส/ชื่อ, กรองสถานะ, แบ่งหน้า 25 รายการ)
  - `/db/dbpg004/lookup[/{id}]` ฟอร์มค่าอ้างอิง (`?category=` เลือกหมวดล่วงหน้า; ตั้งชื่อหมวดใหม่ได้)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg004` ([`dbpg004_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg004_router.rs)):
  - `GET /categories`, `GET /summary`
  - `GET|POST /lookups` (`?category=&search=&active=&page=&limit=`), `GET|PUT /lookups/{id}`
  - `DELETE /lookups/{id}` ลบจริงในหน้าแก้ไข (204): ลบ labels ของค่าแล้วลบค่าใน transaction เดียว ลบได้เฉพาะเมื่อไม่มีใครอ้างอิง มิฉะนั้นตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน; ค่าในหมวดที่ระบบป้องกัน (`permission_type`) ลบไม่ได้ (400)
- **Application Layer**: [`features/db/dbpg004/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg004/) (`Dbpg004Queries`, `Dbpg004Commands`, `Dbpg004CommandExecutor`)
- **Business rules ที่บังคับใช้**:
  - `(category, code)` ไม่ซ้ำ (เทียบรหัสแบบไม่สนตัวพิมพ์) หมวดเป็น `a-z 0-9 _` รหัสเก็บเป็นตัวพิมพ์ใหญ่
  - หมวดและรหัสแก้ไขภายหลังไม่ได้ (ตารางอื่นและโค้ดอ้างอิงด้วยรหัสเหล่านี้) แก้ได้เฉพาะชื่อไทย/อังกฤษและสถานะ
  - หมวด `permission_type` (READ/WRITE/APPROVE) เป็นค่าหลักของระบบ: เพิ่มค่าใหม่ไม่ได้ และปิดใช้งานไม่ได้ แก้ชื่อได้
  - ชื่อไทยบังคับ (อังกฤษเว้นว่างได้: สร้างใหม่คัดลอกชื่อไทย แก้ไขไม่แตะ) ตรวจ `row_version` และเขียนค่า+ชื่อใน transaction เดียว
- **ยังไม่ได้ทำ**: ลบค่าอ้างอิง (ยังไม่มี; ใช้ปิดใช้งานแทน), ลำดับการแสดงผล / ค่าเริ่มต้น (ตาราง `db_master_lookup` ไม่มีคอลัมน์รองรับ), สกุลเงิน (`db_currency`), ล้าง Cache ใน Valkey เมื่อแก้ไข (ปัจจุบันยังไม่มี cache ของ lookup), ตรวจว่าค่านั้นถูกใช้งานอยู่ก่อนปิดใช้งาน
- **Domain Layer**: `domain::entity::{db_master_lookup, db_master_lookup_label}`
