---
id: "019ca1ed-0000-7000-8003-000000000012"
code: "DBPG002"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg002"
icon: "BankOutlined"
title_th: "กำหนดสาขา / แผนก"
title_en: "Branches & Departments"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - branch
  - department
  - screen
---

# 🏛️ DBPG002 - กำหนดสาขา / แผนก (Branches & Departments)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG002`
> - **ชื่อภาษาไทย**: กำหนดสาขา / แผนก
> - **Title (EN)**: Branches & Departments
> - **Route Path**: `/db/dbpg002`
> - **Icon**: `BankOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Parent Folder**: `DBPG000` (ข้อมูลหลัก / Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000012`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการโครงสร้างหน่วยงานย่อยภายใต้บริษัท ได้แก่ สาขา (`db_branch`) สำนักงานสาขา คลังสินค้า และแผนกงาน (`db_department`) เพื่อใช้กำหนดขอบเขตการทำงาน (Data Scoping / Cost Center) และการเข้าถึงข้อมูลของพนักงาน

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น กำหนดสาขาสำนักงานใหญ่/สาขาย่อย, กำหนดแผนกงานและผังบังคับบัญชา]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ข้อมูลสาขา (Branch)**:
  - รหัสสาขา (Branch Code): เช่น `HO`, `B001`
  - ชื่อสาขาภาษาไทย / อังกฤษ: `db_branch_label`
  - บริษัทต้นสังกัด: เลือกจาก `db_company`
  - สำนักงานใหญ่ (Is Headquarter): boolean
  - สถานที่ตั้งและที่อยู่: เชื่อมโยงกับ `db_address`
- [ ] **ข้อมูลแผนก (Department)**:
  - รหัสแผนก (Department Code): เช่น `IT`, `HR`, `ACC`
  - ชื่อแผนกภาษาไทย / อังกฤษ: `db_department_label`
  - แผนกแม่/สังกัด (Parent Department): รองรับ Hierarchical Tree
  - สาขาที่สังกัด: เชื่อมโยงกับ `db_branch`

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Branch Tree View**: แสดงโครงสร้างสาขาและแผนกในรูปแบบ Tree Hierarchy
- [ ] **Headquarter Rule**: บริษัทหนึ่งแห่งต้องมีสาขาสำนักงานใหญ่ (Headquarter) ได้เพียงแห่งเดียวเท่านั้น
- [ ] **Validation**: ตรวจสอบการอ้างอิงพนักงานก่อนปิดการใช้งาน (Deactivate)

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ดูข้อมูลสาขาและแผนก
  - `WRITE`: เพิ่ม แก้ไข โอนย้ายโครงสร้างสาขาและแผนก
  - `APPROVE`: ลบข้อมูลสาขาและแผนก
- **การตรวจสิทธิ์ที่ API (program `DBPG002`)**: ทุก endpoint ต้องมีสิทธิ์ตามโปรแกรม `DBPG002` — `READ` = ดู/ค้นหา (ทุก GET), `WRITE` = เพิ่ม/แก้ไข (POST/PUT), `APPROVE` = ลบ (DELETE); ไม่มีสิทธิ์จะได้ 403
- **Data Scoping**: กรองการมองเห็นสาขาตามสิทธิ์ `branch_id` ของผู้ใช้งาน

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_branch` (id, company_id, code, is_headquarter, active, ...)
- `db_branch_label` (branch_id, language_id, label)
- `db_department` (id, branch_id, department_id, code, active, ...)
- `db_department_label` (department_id, language_id, label)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg002/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg002/) (`Dbpg002Page`, `Dbpg002BranchPage`, `Dbpg002DepartmentPage`)
  - `/db/dbpg002` รายการสาขา/แผนก (แท็บ + ตัวกรองบริษัท/ค้นหา/สถานะ, `?company_id=` เลือกบริษัทล่วงหน้า)
  - `/db/dbpg002/branch[/{id}]` ฟอร์มสาขา (`?company_id=`), `/db/dbpg002/department[/{id}]` ฟอร์มแผนก (`?branch_id=`, `?company_id=`)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg002` ([`dbpg002_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg002_router.rs)):
  - `GET /companies`, `GET /summary`, `GET /branches/types`
  - `GET|POST /branches`, `GET|PUT /branches/{id}`
  - `GET|POST /departments`, `GET|PUT /departments/{id}`
  - `DELETE /branches/{id}` ลบจริงเมื่อไม่มีการอ้างอิง (ลบ label และ address/contact group ของสาขาไปด้วยใน transaction เดียว; สำนักงานใหญ่ลบไม่ได้ → 400; ถ้ายังมีแผนก/พนักงานอ้างอิง → 409 `IN_USE` ให้ปิดใช้งานแทน)
  - `DELETE /departments/{id}` ลบจริงเมื่อไม่มีการอ้างอิง (ลบ label ของแผนกไปด้วย; ถ้ายังถูกอ้างอิง → 409 `IN_USE` ให้ปิดใช้งานแทน)
- **Application Layer**: [`features/db/dbpg002/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg002/) (`Dbpg002Queries`, `Dbpg002Commands`, `Dbpg002CommandExecutor`)
- **Business rules ที่บังคับใช้**: รหัสสาขาไม่ซ้ำภายในบริษัท, รหัสแผนกไม่ซ้ำภายในสาขา, สำนักงานใหญ่ (`branch_type = HEAD`) ได้ 1 สาขาต่อบริษัท, optimistic lock ด้วย `row_version`, สร้างสาขาจะสร้าง `db_address_group` / `db_contact_group` ให้อัตโนมัติ (ภายใน transaction เดียว), แผนกอิงองค์กร/บริษัทตามสาขา, บริษัทของสาขาแก้ไขภายหลังไม่ได้
- **ยังไม่ได้ทำ**: จัดการที่อยู่/ช่องทางติดต่อของสาขา, แผนกแม่ (ไม่มีคอลัมน์ใน `db_department`), ตรวจการอ้างอิงพนักงานก่อนปิดใช้งาน, Tree View
- **Domain Layer**: `domain::entity::{db_branch, db_branch_label, db_department, db_department_label}`
