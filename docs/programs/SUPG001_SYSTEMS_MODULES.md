---
id: "019ca1ed-0000-7000-8003-000000000021"
code: "SUPG001"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg001"
icon: "ClusterOutlined"
title_th: "จัดการระบบ / โมดูล"
title_en: "Systems & Modules"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - system
  - module
  - screen
---

# 🧩 SUPG001 - จัดการระบบ / โมดูล (Systems & Modules)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SUPG001`
> - **ชื่อภาษาไทย**: จัดการระบบ / โมดูล
> - **Title (EN)**: Systems & Modules
> - **Route Path**: `/su/supg001`
> - **Icon**: `ClusterOutlined` (`icondata_ai`)
> - **System**: `SU` (ความปลอดภัย / Security)
> - **Module**: `SUMT` (จัดการสิทธิ์ / Security Maintenance)
> - **Parent Folder**: `SUPG000` (กำหนดสิทธิ์ / Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000021`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับผู้ดูแลระบบ (System Administrator) ในการลงทะเบียนและบำรุงรักษาโครงสร้างระดับสูงของ Application ได้แก่ รายการระบบย่อย (`db_system`) และโมดูลการทำงาน (`db_module`) เพื่อเป็นขอบเขตในการแบ่งสิทธิ์ (Scope) และจัดกลุ่มเมนู

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น เพิ่ม System ใหม่ (เช่น Accounting, Inventory), เพิ่ม Module ย่อย]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ระบบ (System)**:
  - รหัสระบบ (System Code): เช่น `SY`, `DB`, `SU`, `AL`
  - ชื่อระบบภาษาไทย / อังกฤษ: `db_system_label`
  - สถานะเปิดใช้งาน: `active`
- [ ] **โมดูล (Module)**:
  - รหัสโมดูล (Module Code): เช่น `SYMT`, `DBMT`, `SUMT`, `ALMT`
  - ระบบที่สังกัด: เชื่อมโยงกับ `db_system.id`
  - ชื่อโมดูลภาษาไทย / อังกฤษ: `db_module_label`
  - สถานะเปิดใช้งาน: `active`

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **System Code Immutability**: รหัสระบบเมื่อสร้างแล้วห้ามแก้ไข เพราะใช้อ้างอิงใน Permission Index และ Route
- [ ] **Cascade Active Toggle**: หากปิดการใช้งาน System ระบบต้องปิดการแสดงผลของ Module และ Program ภายใต้ System นั้นอัตโนมัติ

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ดูรายการระบบและโมดูล
  - `WRITE`: เพิ่มหรือแก้ไขชื่อระบบ/โมดูล
  - `APPROVE`: อนุมัติการเปิดใช้งานโมดูลใหม่
  - การลบระบบ/โมดูล (DELETE) ต้องมีสิทธิ์ `APPROVE` (ลบ = APPROVE)
- **Restricted Access**: สำหรับผู้ดูแลระบบกลุ่ม `ADMIN` เท่านั้น

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_system` (id, code, active, ...)
- `db_system_label` (system_id, language_id, label)
- `db_module` (id, system_id, code, active, ...)
- `db_module_label` (module_id, language_id, label)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `handler/web/application/master/src/pages/su/supg001/` (แท็บระบบ/โมดูล, `UnitForm`) • routes `/su/supg001`, `/su/supg001/{system|module}[/:id]`
- **Backend API** (`/api/v1/su/supg001`, ต้องผ่าน Kratos session; READ = GET, WRITE = POST/PUT): `GET|POST /systems`, `PUT /systems/{id}`, `GET|POST /modules`, `PUT /modules/{id}`
- **Application**: `application/master/src/features/su/supg001/` (รหัส 2–10 ตัว A-Z0-9 ตัวพิมพ์ใหญ่; แก้ไขรหัสไม่ได้)
- **ลบ**: `DELETE /systems/{id}` และ `DELETE /modules/{id}` (WRITE, 204) ลบจริงพร้อม label ใน transaction เดียว; `SU`/`SUMT` ลบไม่ได้ (Validation), ถ้ายังมีโมดูล/โปรแกรม/สิทธิ์อ้างอิงอยู่ DB ปฏิเสธ (FK) → 409 `IN_USE` ให้ปิดใช้งานแทน
- **กฎสำคัญ**: ระบบ `SU` และโมดูล `SUMT` ปิดใช้งานไม่ได้ (กัน lock-out)
- **Tests**: `application/master/tests/supg001_test.rs`, `handler/api/master/tests/supg001_router_test.rs`
