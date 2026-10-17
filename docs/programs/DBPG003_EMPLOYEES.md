---
id: "019ca1ed-0000-7000-8003-000000000013"
code: "DBPG003"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg003"
icon: "IdcardOutlined"
title_th: "ทะเบียนพนักงาน"
title_en: "Employees"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - employee
  - hr
  - screen
---

# 👤 DBPG003 - ทะเบียนพนักงาน (Employees)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG003`
> - **ชื่อภาษาไทย**: ทะเบียนพนักงาน
> - **Title (EN)**: Employees
> - **Route Path**: `/db/dbpg003`
> - **Icon**: `IdcardOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Parent Folder**: `DBPG000` (ข้อมูลหลัก / Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000013`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการข้อมูลประวัติพนักงาน (`db_employee`) รายละเอียดชื่อ-นามสกุลหลายภาษา (`db_person_name`), ตำแหน่งงาน, สังกัดแผนก/สาขา, ข้อมูลติดต่อส่วนบุคคล และการเชื่อมโยงกับบัญชีผู้ใช้งานระบบ (`su_user`)

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น จัดการข้อมูลพนักงาน, กำหนดสังกัดแผนก, เชื่อมโยงบัญชี Login]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ข้อมูลส่วนตัวพนักงาน**:
  - รหัสพนักงาน (Employee Code): `code`
  - คำนำหน้า / ชื่อ / นามสกุล (TH/EN): `db_person_name`, `db_person_name_label`
  - หมายเลขบัตรประชาชน / Passport
  - วันเกิด / วันเริ่มงาน / วันพ้นสภาพ
- [ ] **ข้อมูลสังกัดและตำแหน่งงาน**:
  - บริษัทต้นสังกัด (`db_company`)
  - สาขาที่ปฏิบัติงาน (`db_branch`)
  - แผนกงาน (`db_department`)
  - ตำแหน่งงาน (Position / Job Title)
- [ ] **การผูกกับ User Account**:
  - เชื่อมโยงกับ `su_user.id` สำหรับพนักงานที่มีสิทธิ์ใช้งานระบบ ERP

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Employee Search & Filter**: ค้นหาตามรหัส, ชื่อ-นามสกุล, แผนก, สาขา หรือสถานะการทำงาน
- [ ] **Auto Code Generator**: รองรับการสร้างรหัสพนักงานอัตโนมัติตามรูปแบบที่กำหนด
- [ ] **Termination Workflow**: เมื่อพนักงานพ้นสภาพ ระบบต้องระงับบัญชีผู้ใช้ `su_user` และ Session Kratos อัตโนมัติ

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ดูข้อมูลพนักงานในสาขา/แผนกที่ตนเองมีสิทธิ์
  - `WRITE`: เพิ่มและแก้ไขข้อมูลประวัติพนักงาน
  - `APPROVE`: ลบข้อมูลพนักงาน
- **การตรวจสิทธิ์ที่ API (program `DBPG003`)**: ทุก endpoint ต้องมีสิทธิ์ตามโปรแกรม `DBPG003` — `READ` = ดู/ค้นหา (ทุก GET), `WRITE` = เพิ่ม/แก้ไข (POST/PUT), `APPROVE` = ลบ (DELETE); ไม่มีสิทธิ์จะได้ 403
- **Data Privacy**: ปิดบังข้อมูลส่วนบุคคลที่อ่อนไหว (เช่น บัตรประชาชน, บัญชีธนาคาร) ตาม PDPA

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_employee` (id, company_id, branch_id, department_id, person_name_id, code, active, ...)
- `db_person_name` (id, active)
- `db_person_name_label` (person_name_id, language_id, prefix, first_name, last_name, ...)
- `db_contact_group`, `db_contact`
- `su_user` (เชื่อมโยงผ่าน `user_id` / `employee_id`)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg003/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg003/) (`Dbpg003Page`, `Dbpg003EmployeePage`)
- **ปิดใช้งานพนักงาน = ระงับบัญชี**: เมื่อบันทึกพนักงานเป็น inactive ระบบระงับบัญชีผู้ใช้ (`su_user`) ที่ผูกกับพนักงานทุกบัญชีทันที (ปิด identity ที่ Kratos + เพิกถอน session + ปิดแฟล็กในระบบ) ถ้าระงับไม่สำเร็จจะแจ้งข้อผิดพลาดและบันทึกซ้ำเพื่อลองใหม่ได้; ตัวตรวจตอนล็อกอิน (login guard) ปฏิเสธพนักงานที่ inactive ด้วย; การเปิดพนักงานกลับเป็น active (จาก inactive) จะเปิดบัญชีที่ผูกกับพนักงานให้อัตโนมัติ (เปิด identity + แฟล็ก) — ทำเฉพาะตอนเปลี่ยนสถานะ จึงไม่ล้มการระงับที่ผู้ดูแลทำไว้เองเมื่อบันทึกพนักงานที่ active อยู่แล้ว; ถ้าเปิดไม่สำเร็จให้เปิดเองที่ SUPG004/SUPG005
  - `/db/dbpg003` รายการพนักงาน (กรองบริษัท/สาขา, ค้นหารหัส/ชื่อ/นามสกุล/ชื่อเล่น, กรองสถานะ, แบ่งหน้า 25 รายการ)
  - `/db/dbpg003/employee[/{id}]` ฟอร์มพนักงาน (`?company_id=` เลือกบริษัทล่วงหน้า) เลือกบริษัท → สาขา → แผนก แบบต่อเนื่อง
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg003` ([`dbpg003_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg003_router.rs)):
  - `GET /companies`, `/branches`, `/departments`, `/prefixes`, `/summary`
  - `GET|POST /employees` (`?page=&limit=&search=&active=&company_id=&branch_id=&department_id=`), `GET|PUT /employees/{id}`
  - **รูปพนักงาน** (`db_employee.image_key`, ไฟล์ใน Garage S3 ที่ `images/employees/{id}/<uuidv7>.<ext>`): `GET|PUT|DELETE /employees/{id}/image` — `PUT` รับไฟล์ดิบเป็น body ตอบ DTO ที่มี `image_url`; `GET` ต้อง Read, `PUT`/`DELETE` ใช้สิทธิ์ Write แบบเดียวกับการแก้ไข (จำกัดตามขอบเขตแผนกของพนักงาน) กฎไฟล์ (PNG/JPEG/GIF/WebP ≤ 2 MB ตัดสินจากเนื้อไฟล์ ไม่รับ SVG) และการทำงานเหมือนโลโก้ org/company ดู [[DBPG001_ORGANIZATIONS]] ตัวเก็บรูปกลางคือ `application/master/src/features/db/image_object_store.rs`; เว็บใช้ `web_package::ui::component::ImageField`
  - `DELETE /employees/{id}` ลบจริงในหน้าแก้ไข (204): ลบ `db_person_name` + labels ที่เป็นของพนักงานแล้วลบพนักงานใน transaction เดียว ลบได้เฉพาะเมื่อไม่มีใครอ้างอิง (เช่น มีบัญชี `su_user` ให้ลบที่ SUPG004 ก่อน) มิฉะนั้นตอบ 409 `IN_USE` → ให้ปิดใช้งานแทน
- **Application Layer**: [`features/db/dbpg003/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg003/) (`Dbpg003Queries`, `Dbpg003Commands`, `Dbpg003CommandExecutor`)
- **Business rules ที่บังคับใช้**: รหัสพนักงานไม่ซ้ำภายในแผนก, ชื่อ/นามสกุลไทยบังคับ (อังกฤษเว้นว่างได้: สร้างใหม่จะคัดลอกชื่อไทย, แก้ไขจะไม่แตะ), คำนำหน้าต้องเป็น lookup `prefix`, องค์กร/บริษัท/สาขาอิงตามแผนกที่เลือก, optimistic lock ด้วย `row_version`, เขียน `db_person_name` + labels + `db_employee` ใน transaction เดียว
- **Query**: กรอง/ค้นหา/แบ่งหน้าทำใน SQL และโหลด label เฉพาะหน้าที่แสดง
- **ยังไม่ได้ทำ**: เลขบัตรประชาชน/วันเกิด/วันเริ่มงาน/ตำแหน่งงาน (ตาราง `db_employee` ไม่มีคอลัมน์รองรับ), ข้อมูลติดต่อและที่อยู่ (`db_contact_group` / `db_address_group` เป็น optional), Auto Code Generator, Termination Workflow (ระงับ `su_user` + Kratos อัตโนมัติเมื่อพ้นสภาพ), การปิดบังข้อมูลอ่อนไหวตาม PDPA
- **Domain Layer**: `domain::entity::{db_employee, db_person_name, db_person_name_label}`
