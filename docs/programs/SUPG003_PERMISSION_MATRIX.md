---
id: "019ca1ed-0000-7000-8003-000000000023"
code: "SUPG003"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg003"
icon: "SafetyOutlined"
title_th: "แฟ้มสิทธิ์การเข้าใช้งาน"
title_en: "Permission Matrix"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - permissions
  - rbac
  - abac
  - screen
---

# 🛡️ SUPG003 - แฟ้มสิทธิ์การเข้าใช้งาน (Permission Matrix)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SUPG003`
> - **ชื่อภาษาไทย**: แฟ้มสิทธิ์การเข้าใช้งาน
> - **Title (EN)**: Permission Matrix
> - **Route Path**: `/su/supg003`
> - **Icon**: `SafetyOutlined` (`icondata_ai`)
> - **System**: `SU` (ความปลอดภัย / Security)
> - **Module**: `SUMT` (จัดการสิทธิ์ / Security Maintenance)
> - **Parent Folder**: `SUPG000` (กำหนดสิทธิ์ / Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000023`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำคัญสำหรับการบริหารจัดการความปลอดภัยและการอนุญาตเข้าถึง (Authorization Management) จัดการกลุ่มสิทธิ์ (`su_permission_group`), กำหนดสิทธิ์แบบ Matrix Table ตามโปรแกรมและ Action (`READ`, `WRITE`, `APPROVE`) และกำหนดขอบเขตข้อมูลแบบ ABAC Scope (`company_id`, `branch_id`, `department_id`)

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น สร้างกลุ่มสิทธิ์ (เช่น HR_MANAGER, ACCOUNTANT), ติ๊กเลือกสิทธิ์ในตาราง Matrix]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **กลุ่มสิทธิ์ (Permission Group)**:
  - รหัสกลุ่มสิทธิ์ (Group Code): เช่น `ADMIN`, `SUPV`, `OPERATOR`
  - ชื่อกลุ่มสิทธิ์ภาษาไทย / อังกฤษ
  - คำอธิบายหน้าที่ความรับผิดชอบ
- [ ] **ตารางเมทริกซ์สิทธิ์ (Permission Matrix Grid)**:
  - แกนแถว (Rows): รายการ System -> Module -> Program (Tree View)
  - แกนคอลัมน์ (Columns): สิทธิ์การทำงาน (`READ`, `WRITE`, `APPROVE`)
  - Checkboxes: ติ๊กเปิด/ปิดสิทธิ์ในแต่ละช่อง
- [ ] **การกำหนดขอบเขตข้อมูล (ABAC Scope Dimensions)**:
  - Scope ระดับบริษัท (Company Scope)
  - Scope ระดับสาขา (Branch Scope)
  - Scope ระดับแผนก (Department Scope)

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Admin Group Protection**: กลุ่ม `ADMIN` มีสิทธิ์เต็มทุกโปรแกรมโดยอัตโนมัติ (ห้ามถอดสิทธิ์ตนเองจนหมด)
- [ ] **Bulk Grant / Revoke**: มีปุ่ม "เลือกทั้งหมดในโมดูล" (Select All in Module) เพื่อความสะดวกรวดเร็ว
- [ ] **Audit Trail on Changes**: ทุกครั้งที่มีการเปลี่ยนแปลงสิทธิ์ ต้องบันทึกเหตุการณ์การแก้ไขลงใน Quickwit Audit Log อย่างละเอียด

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: เข้าดูตารางกลุ่มสิทธิ์และเมทริกซ์
  - `WRITE`: แก้ไขสิทธิ์ เพิ่มกลุ่มสิทธิ์ใหม่
  - `APPROVE`: อนุมัติการเผยแพร่กลุ่มสิทธิ์ใหม่สู่ผู้ใช้งานจริง
  - การลบกลุ่มสิทธิ์ (DELETE) ต้องมีสิทธิ์ `APPROVE` (ลบ = APPROVE)

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `su_permission_group` (id, code, active, ...)
- `su_permission` (id, permission_group_id, system_id, module_id, program_id, master_lookup_id, active, ...)
- `su_user_permission_group` (เชื่อม User เข้ากับ Group)
- `db_master_lookup` (หมวด `permission_type` สำหรับ action: READ, WRITE, APPROVE)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg003/` (รายการกลุ่ม, `GroupForm`, `MatrixTable` แบบ checkbox + เลือกทั้งหมด) • routes `/su/supg003`, `/su/supg003/group[/:id]`, `/su/supg003/group/:id/matrix`
- **Backend API** (`/api/v1/su/supg003`): `GET|POST /groups`, `GET|PUT /groups/{id}`, `GET|PUT /groups/{id}/matrix`
- **Application**: `features/su/supg003/` (`plan_changes` เทียบสิทธิ์เดิม-ใหม่, ปิดแถวแทนการลบ; WRITE/APPROVE ต้องมี READ)
- **ลบกลุ่ม**: `DELETE /groups/{id}` (WRITE → 204) ลบจริงพร้อมแถว `su_permission` ของกลุ่มใน transaction เดียว; กลุ่ม `ADMIN` ลบไม่ได้ (400), กลุ่มที่ยังมีผู้ใช้หรือถูกอ้างอิงอยู่ → 409 `IN_USE` (ให้ย้ายผู้ใช้ก่อน / ปิดใช้งานแทน)
- **กฎสำคัญ**: กลุ่ม `ADMIN` แก้เมทริกซ์/ปิดใช้งานไม่ได้; ห้ามถอดสิทธิ์ที่ทำให้ผู้บันทึกเข้า SUPG003 ไม่ได้; ปิดกลุ่มที่ยังมีผู้ใช้ไม่ได้
- **กลุ่มสิทธิ์ไม่มีขอบเขต**: `su_permission_group` เก็บแค่ `code`, `active` (รหัสกลุ่มไม่ซ้ำ) = ชุดสิทธิ์ "ทำอะไรได้" (โปรแกรม × READ/WRITE/APPROVE) ส่วน "ทำที่ไหน" (org / company / branch / department) กำหนดต่อผู้ใช้ที่หน้า SUPG004 ผ่าน `su_user_assignment` (ผู้ใช้ 1 คนถือได้หลายกลุ่มหลายขอบเขต เช่น กลุ่ม A ที่สาขา 1 และกลุ่ม B ที่สาขา 2 ได้ "สาขา 1 ทำ 1,2,3 / สาขา 2 ทำ 1,2,6") • กลุ่ม `ADMIN` ปิดใช้งานหรือลบไม่ได้
- **ตรวจสิทธิ์ตามข้อมูลที่แตะ**: `AccessControl::authorize_in(user, program, action, ScopeTarget)` — ต้องมี assignment ที่กลุ่มให้สิทธิ์นั้น และขอบเขตของ assignment ครอบคลุมหน่วยที่แตะ (ขอบเขตสูงครอบคลุมหน่วยใต้มัน; ผู้ใช้ที่ไม่มีแถว assignment ถือว่ากลุ่มหลักใช้ได้ทุกที่) ใช้แล้วกับการเขียน/ลบของ DBPG002 (สาขา = ตามบริษัท, แผนก = ตามสาขา) และ DBPG003 (พนักงาน = ตามแผนก, ย้ายแผนกต้องมีสิทธิ์ทั้งสองฝั่ง) • การอ่าน (รายการ) และเมนูฝั่งเว็บดูสิทธิ์รวมทุก assignment และรายการข้อมูลยังไม่ถูกกรองตามขอบเขต
- **หมายเหตุ**: การบันทึกเมทริกซ์เป็น last-write-wins
- **ยังไม่บังคับใช้**: ขอบเขตของ assignment ยังไม่ได้กรองรายการข้อมูล (GET) ใน API อื่น
- **Tests**: `supg003_test.rs`, `supg003_router_test.rs`
