---
id: "019ca1ed-0000-7000-8003-000000000024"
code: "SUPG004"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg004"
icon: "UserOutlined"
title_th: "กำหนดผู้ใช้งาน"
title_en: "User Accounts"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - iam
  - users
  - ory-kratos
  - screen
---

# 👥 SUPG004 - กำหนดผู้ใช้งาน (User Accounts)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SUPG004`
> - **ชื่อภาษาไทย**: กำหนดผู้ใช้งาน
> - **Title (EN)**: User Accounts
> - **Route Path**: `/su/supg004`
> - **Icon**: `UserOutlined` (`icondata_ai`)
> - **System**: `SU` (ความปลอดภัย / Security)
> - **Module**: `SUMT` (จัดการสิทธิ์ / Security Maintenance)
> - **Parent Folder**: `SUPG000` (กำหนดสิทธิ์ / Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000024`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอสำหรับจัดการบัญชีผู้ใช้งานระบบ ERP (`su_user`), กำหนดกลุ่มสิทธิ์ (`su_permission_group`), กำหนดขอบเขตข้อมูลรายบุคคล (Organization/Branch Scopes), จัดการสถานะระงับการใช้งาน (Lock/Unlock) และประสานงานกับระบบ **Ory Kratos 1.3** ในการสร้าง Identity และส่งลิงก์ตั้งค่ารหัสผ่าน (Recovery Link)

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น สร้าง User ใหม่, มอบหมาย Role, รีเซ็ตรหัสผ่าน, ปิดการใช้งานบัญชี]*

### 2.2 ข้อมูลนำเข้าและฟิลด์ที่เกี่ยวข้อง (Fields & Schema)
- [ ] **ข้อมูลบัญชี (Account Info)**:
  - ชื่อผู้ใช้ (Username): อีเมลหรือชื่อผู้ใช้สำหรับเข้าสู่ระบบ
  - อีเมล (Email): ใช้สำหรับแจ้งเตือนและ Recovery ทาง Ory Kratos
  - ชื่อ-นามสกุลผู้ใช้งาน: ภาษาไทย / อังกฤษ
  - พนักงานต้นสังกัด: เลือกผูกกับ `db_employee` (ถ้าเป็นพนักงานภายใน)
  - ภาษาเริ่มต้น (Preferred Language): TH / EN
- [ ] **การกำหนดสิทธิ์และขอบเขต (Role & Scope Assignment)**:
  - กลุ่มสิทธิ์ (Permission Groups): เลือกได้หลายกลุ่ม (Multi-roles)
  - ขอบเขตข้อมูล (Data Scope): บริษัท, สาขา, แผนก ที่อนุญาตให้เข้าถึง
- [ ] **สถานะความปลอดภัย (Security Status)**:
  - สถานะเปิดใช้งาน: `active` (boolean)
  - สภาพการล็อกบัญชี: Lockout status
  - การเปิดใช้งาน MFA (2FA Status): ผ่าน Ory Kratos TOTP/WebAuthn

### 2.3 การทำงานและเงื่อนไขทางธุรกิจ (Business Rules & Actions)
- [ ] **Ory Kratos Sync**: เมื่อสร้างบัญชีใหม่ Backend เรียก Ory Kratos Admin API สร้าง Identity พร้อม Schema Traits
- [ ] **Password Reset**: ไม่เก็บ Plain Password หรือจัดการเอง แต่สั่ง Kratos ส่ง Recovery Flow / One-time Link ไปยังอีเมลของผู้ใช้
- [ ] **Session Revocation**: เมื่อสั่งระงับบัญชี (Deactivate) ให้สั่ง Kratos Revoke Active Sessions ทันที

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: ค้นหาและดูรายชื่อผู้ใช้งาน
  - `WRITE`: สร้างผู้ใช้งานใหม่, แก้ไขข้อมูล, มอบหมายกลุ่มสิทธิ์
  - `APPROVE`: อนุมัติการสร้าง Super Admin หรือขยายสิทธิ์ระดับสูง
  - การลบผู้ใช้งาน (DELETE) ต้องมีสิทธิ์ `APPROVE` (ลบ = APPROVE) ; การจัดบัญชีเข้ากลุ่ม ADMIN ก็ใช้ `APPROVE` เช่นกัน

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- `su_user` (id, username, email, active, created_at, ...)
- `su_user_permission_group` (user_id, permission_group_id, ...)
- `db_employee` (เชื่อมโยงประวัติพนักงาน)
- Ory Kratos `identities` table (จัดการใน Kratos Database)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg004/` (ตารางบัญชี server-paged, `UserForm` ค้นหาพนักงาน/อีเมล/username/กลุ่ม, `RecoveryLinkPanel`) • routes `/su/supg004`, `/su/supg004/user[/:id]`
- **Backend API** (`/api/v1/su/supg004`): `GET /users`, `GET|PUT /users/{id}`, `POST /users` (สร้าง Kratos identity + แถว `su_user` + ลิงก์ตั้งรหัสผ่าน), `POST /users/{id}/recovery-link`, `GET /options/employees`, `GET /options/groups`
- **Application**: `features/su/supg004/` — `IdentityProvisioningPort` (`application/core`) ถูก implement โดย `KratosConfig` (Kratos Admin API: `/admin/identities`, `/admin/recovery/link`)
- **สิทธิ์ต่อขอบเขต (assignments)**: ผู้ใช้ 1 คนมีได้หลาย assignment (`su_user_assignment`) แต่ละอัน = กลุ่มสิทธิ์ 1 กลุ่ม ที่ขอบเขต 1 หน่วย (organization / company / branch / department) เช่น กลุ่ม A ที่สาขา 1 + กลุ่ม B ที่สาขา 2 → ได้ "สาขา 1 ทำ 1,2,3 / สาขา 2 ทำ 1,2,6" โดยกลุ่มไม่ต้องรู้ขอบเขต (กลุ่มคือชุดสิทธิ์ล้วนๆ ที่ SUPG003) • ต้องมีอย่างน้อย 1 assignment, อันแรกคือกลุ่มหลัก (`su_user.permission_group_id`) • แก้แบบ diff ใน transaction เดียว • ตัวเลือกขอบเขตมาจาก `GET /scope-options` • ใช้ตรวจสิทธิ์จริงที่ `AccessControl::authorize_in` (เขียน/ลบของ DBPG002 / DBPG003), เมนูและ `/access/me` รวมสิทธิ์ทุก assignment
- **ลบบัญชี**: `DELETE /users/{id}` (WRITE → 204) ลบแถว `su_user` จริง แล้วเพิกถอนเซสชันและลบ Kratos identity (best effort, ล้มเหลวแค่ log เตือน); ลบบัญชีตัวเองไม่ได้ (403), ถูกอ้างอิงอยู่ → 409 `IN_USE` (ให้ระงับการใช้งานแทน)
- **กฎสำคัญ**: สร้างได้เฉพาะพนักงานที่ยังไม่มีบัญชี; DB ล้มเหลว → ลบ identity ที่เพิ่งสร้าง (compensating); ย้ายเข้ากลุ่ม ADMIN ต้องมีสิทธิ์ APPROVE; ห้ามระงับ/ย้ายกลุ่มตัวเองจนเข้าโปรแกรมไม่ได้; ระงับ/เปิดใช้งานผ่าน `Supg005Commands` (เพิกถอนเซสชัน)
- **Tests**: `supg004_test.rs`, `supg004_router_test.rs`
