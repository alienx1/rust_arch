---
id: "019ca1ed-0000-7000-8003-000000000015"
code: "DBPG005"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg005"
icon: "GlobalOutlined"
title_th: "จังหวัด / อำเภอ / ตำบล"
title_en: "Geography"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - geography
  - province
  - district
  - screen
---

# 🌍 DBPG005 - จังหวัด / อำเภอ / ตำบล (Geography)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG005`
> - **ชื่อภาษาไทย**: จังหวัด / อำเภอ / ตำบล
> - **Title (EN)**: Geography
> - **Route Path**: `/db/dbpg005`
> - **Icon**: `GlobalOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Parent Folder**: `DBPG000` (ข้อมูลหลัก / Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000015`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการและค้นหาข้อมูลที่ตั้งทางภูมิศาสตร์ ได้แก่ ประเทศ (`db_country`), จังหวัด (`db_province`), อำเภอ/เขต (`db_district`), ตำบล/แขวง (`db_sub_district`) และรหัสไปรษณีย์ (`db_postal_code`) เพื่อใช้ในระบบบันทึกที่อยู่ขององค์กร ลูกค้า และพนักงาน

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น ค้นหารหัสไปรษณีย์อัตโนมัติ (Typeahead), บำรุงรักษาข้อมูลการแบ่งเขตการปกครอง]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ลำดับชั้นทางภูมิศาสตร์ (Hierarchy Cascade)**:
  - ประเทศ (Country): รหัส ISO เช่น `TH`
  - จังหวัด (Province): รหัสจังหวัด, ชื่อภาษาไทย / อังกฤษ
  - อำเภอ / เขต (District): เชื่อมโยงกับ `province_id`
  - ตำบล / แขวง (Sub-District): เชื่อมโยงกับ `district_id`
  - รหัสไปรษณีย์ (Postal Code): ตัวเลข 5 หลัก ผูกกับตำบล
- [ ] **การจัดเรียงภาษาไทย (Thai Collation)**:
  - ใช้ PostgreSQL ICU Collation `th-TH-x-icu` เพื่อเรียงลำดับ ก-ฮ อย่างถูกต้อง

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Cascading Dropdowns**: เมื่อเลือกจังหวัด -> กรองเฉพาะอำเภอในจังหวัดนั้น -> กรองเฉพาะตำบล -> ดึงรหัสไปรษณีย์ขึ้นมาเติมอัตโนมัติ
- [ ] **Postal Code Quick Search**: ค้นหาด้วยรหัสไปรษณีย์ แล้วระบบ Auto-fill ตำบล อำเภอ จังหวัด ให้ทันที

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- API อยู่หลัง Ory Kratos session และตรวจสิทธิ์ด้วย `routers/access.rs::require()` กับโปรแกรม `DBPG005`: `READ` = ดู/ค้นหา (`GET`), `WRITE` = เพิ่ม/แก้ไข (`POST`/`PUT`), `APPROVE` = ลบ (`DELETE`)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ค้นหาและเรียกดูข้อมูลเขตภูมิศาสตร์ (`GET` ทุกเส้นทาง)
  - `WRITE`: เพิ่มหรือแก้ไขจังหวัด อำเภอ ตำบล และรหัสไปรษณีย์ (`POST` / `PUT`)
  - `APPROVE`: ลบข้อมูลเขตภูมิศาสตร์ (`DELETE`)

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_country`, `db_country_label`
- `db_province`, `db_province_label`
- `db_district`, `db_district_label`
- `db_sub_district`, `db_sub_district_label`
- `db_postal_code`

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg005/) (`Dbpg005Page`, `Dbpg005FormPage`)
  - `/db/dbpg005` แท็บ จังหวัด / อำเภอ / ตำบล / รหัสไปรษณีย์ พร้อมตัวกรองจังหวัด → อำเภอ (ต่อเนื่อง), ค้นหา, กรองสถานะ, แบ่งหน้า 25 รายการ ที่ฐานข้อมูล
  - `/db/dbpg005/{province|district|sub-district|postal-code}[/{id}]` ฟอร์มสร้าง/แก้ไข (`?province_id=`, `?district_id=` เลือกพื้นที่ล่วงหน้า) เลือกจังหวัด → อำเภอ → ตำบลแบบต่อเนื่อง
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg005` ([`dbpg005_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg005_router.rs)):
  - `GET /summary`
  - `GET|POST /provinces`, `GET|PUT /provinces/{id}`
  - `GET|POST /districts` (`?province_id=`), `GET|PUT /districts/{id}`
  - `GET|POST /sub-districts` (`?province_id=&district_id=`), `GET|PUT /sub-districts/{id}`
  - `GET|POST /postal-codes` (`?province_id=&district_id=&sub_district_id=&search=`), `GET|PUT /postal-codes/{id}`
  - `DELETE /provinces/{id}` — ลบจริงเมื่อไม่มีใครอ้างอิง (ลบชื่อ label ของตัวเองใน transaction เดียวกัน) ถ้ายังถูกอ้างอิงตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน (มีอำเภอ/รหัสไปรษณีย์/ที่อยู่อ้างอิงอยู่)
  - `DELETE /districts/{id}` — ลบจริงเมื่อไม่มีใครอ้างอิง (ลบชื่อ label ของตัวเองใน transaction เดียวกัน) ถ้ายังถูกอ้างอิงตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน (มีตำบล/รหัสไปรษณีย์/ที่อยู่อ้างอิงอยู่)
  - `DELETE /sub-districts/{id}` — ลบจริงเมื่อไม่มีใครอ้างอิง (ลบชื่อ label ของตัวเองใน transaction เดียวกัน) ถ้ายังถูกอ้างอิงตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน (มีรหัสไปรษณีย์/ที่อยู่อ้างอิงอยู่)
  - `DELETE /postal-codes/{id}` — ลบจริงเมื่อไม่มีที่อยู่อ้างอิง ถ้ายังถูกอ้างอิงตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน
  - ทุกรายการรองรับ `search`, `active`, `page`, `limit` (สูงสุด 200 ต่อหน้า พอสำหรับตัวเลือกทั้งระดับ เช่น 77 จังหวัด)
- **Application Layer**: [`features/db/dbpg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg005/) (`Dbpg005Queries`, `Dbpg005Commands`, `Dbpg005CommandExecutor`)
- **Business rules ที่บังคับใช้**:
  - รหัสไม่ซ้ำ: จังหวัดในประเทศ, อำเภอในจังหวัด, ตำบลในอำเภอ; รหัสไปรษณีย์ไม่ซ้ำในพื้นที่เดียวกัน
  - รหัสและพื้นที่แม่แก้ไขภายหลังไม่ได้ (ที่อยู่และระดับล่างอ้างอิง) แก้ได้เฉพาะชื่อ/สถานะ (รหัสไปรษณีย์แก้ค่าได้)
  - ระดับบนของอำเภอ/ตำบล/รหัสไปรษณีย์ได้จากพื้นที่ที่ละเอียดที่สุดที่เลือก จึงไม่ขัดแย้งกัน; จังหวัดใหม่ผูกกับประเทศที่เปิดใช้งาน
  - ชื่อไทยบังคับ (อังกฤษเว้นว่างได้: สร้างใหม่คัดลอกชื่อไทย แก้ไขไม่แตะ) ตรวจ `row_version` และเขียนข้อมูล+ชื่อใน transaction เดียว
- **ยังไม่ได้ทำ**: ประเทศ (`db_country`; ระบบมีประเทศเดียว `THA`), Postal Code Quick Search แบบ Auto-fill ในฟอร์มที่อยู่ (ต้องรอหน้าจัดการที่อยู่), การเรียงลำดับตัวอักษรไทยด้วย ICU collation `th-TH-x-icu` (ตอนนี้เรียงตามรหัส), ตรวจว่ามีที่อยู่ใช้งานอยู่ก่อนปิดใช้งาน
- **Domain Layer**: `domain::entity::{db_province, db_district, db_sub_district, db_postal_code}` และตาราง label ของแต่ละระดับ
