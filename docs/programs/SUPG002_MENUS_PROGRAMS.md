---
id: "019ca1ed-0000-7000-8003-000000000022"
code: "SUPG002"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg002"
icon: "MenuOutlined"
title_th: "จัดการเมนู"
title_en: "Menus & Programs"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - menu
  - navigation
  - screen
---

# 📑 SUPG002 - จัดการเมนู (Menus & Programs)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SUPG002`
> - **ชื่อภาษาไทย**: จัดการเมนู
> - **Title (EN)**: Menus & Programs
> - **Route Path**: `/su/supg002`
> - **Icon**: `MenuOutlined` (`icondata_ai`)
> - **System**: `SU` (ความปลอดภัย / Security)
> - **Module**: `SUMT` (จัดการสิทธิ์ / Security Maintenance)
> - **Parent Folder**: `SUPG000` (กำหนดสิทธิ์ / Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000022`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการโครงสร้างเมนูนำทาง (Sidebar Navigation Menu) และลงทะเบียนโปรแกรม (`db_program`) จัดการความสัมพันธ์แบบ Hierarchy (Folders & Leaf Programs), การตั้งค่า Route Path, การเลือกไอคอน (`icondata_ai`), การจัดลำดับเมนู (Sort Order) และการคิวรี Menu Tree ตามสิทธิ์ผู้ใช้ (`get_menu_tree`)

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น ปรับแต่งลำดับเมนู, กำหนดโฟลเดอร์แม่-ลูก, อัปเดตไอคอน]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ข้อมูลโปรแกรมและเมนู (Program Details)**:
  - รหัสโปรแกรม (Program Code): เช่น `DBPG001`, `SUPG001`
  - ประเภท: โฟลเดอร์ (Folder: `path` เป็น NULL) หรือ หน้าจอใช้งาน (`path` ระบุเส้นทาง SPA)
  - โมดูลต้นสังกัด (`module_id`) และ ระบบต้นสังกัด (`system_id`)
  - โฟลเดอร์แม่ (`program_id` / Parent Program): Self-referencing FK
  - ชื่อเมนูแสดงผล (Labels): ภาษาไทยและภาษาอังกฤษ (`db_program_label`)
  - ไอคอน (Icon Name): เช่น `DashboardOutlined`, `ApartmentOutlined`
  - ลำดับการแสดง (Sort Order): ตัวเลขจำนวนเต็ม
  - สถานะเปิดใช้งาน: `active` (boolean)

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Drag & Drop Reordering**: รองรับการลากสลับลำดับเมนู หรือย้ายเข้าไปอยู่ใต้โฟลเดอร์อื่น
- [ ] **ABAC Filtering Integration**: เมื่อ Frontend รัน `get_menu_tree` เมนูที่ส่งกลับต้องถูกกรองเฉพาะโปรแกรมที่ User มีสิทธิ์ `READ` เท่านั้น
- [ ] **Ancestor Restoration**: หาก User มีสิทธิ์ในหน้าจอย่อย ระบบต้องดึงโฟลเดอร์บรรพบุรุษขึ้นมาแสดงให้ครบอัตโนมัติ

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ดูโครงสร้างผังเมนู
  - `WRITE`: เพิ่ม แก้ไข จัดลำดับเมนูและโปรแกรม
  - `APPROVE`: อนุมัติการเผยแพร่เมนูใหม่สู่ Production
  - การลบโปรแกรม (DELETE) ต้องมีสิทธิ์ `APPROVE` (ลบ = APPROVE)

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `db_program` (id, system_id, module_id, program_id, code, path, icon, sort_order, active, ...)
- `db_program_label` (program_id, language_id, label)
- `su_permission` (เชื่อมโยงเพื่อตรวจสอบสิทธิ์การเข้าถึงเมนู)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg002/` (ตารางโปรแกรม server-paged + ตัวกรองระบบ/โมดูล/สถานะ, `ProgramForm`) • routes `/su/supg002`, `/su/supg002/program[/:id]`
- **Backend API** (`/api/v1/su/supg002`): `GET /menu` (เมนู sidebar ตามสิทธิ์), `GET /options`, `GET|POST /programs`, `GET|PUT /programs/{id}` (READ/WRITE)
- **Application**: `features/su/supg002/` (`build_menu_tree`: โฟลเดอร์ปิด = ซ่อนทั้งกลุ่ม; path ถูกแปลงเป็นตัวพิมพ์เล็กและขึ้นต้นด้วย `/`; ตรวจโฟลเดอร์แม่ในระบบเดียวกัน/ไม่เป็นวงวน)
- **ลบ**: `DELETE /programs/{id}` (WRITE, 204) ลบจริงพร้อม label และสิทธิ์ (`su_permission`) ของโปรแกรมนั้นใน transaction เดียว; `SUPG001/002/003` ลบไม่ได้ (Validation), ถ้ายังมีโปรแกรมลูกอ้างอิงอยู่ → 409 `IN_USE` ให้ปิดใช้งานแทน
- **กฎสำคัญ**: `SUPG001/002/003` ซ่อนไม่ได้; ระบบ/โมดูล/รหัสแก้ไขไม่ได้หลังสร้าง
- **Tests**: `supg002_test.rs`, `supg002_router_test.rs`
