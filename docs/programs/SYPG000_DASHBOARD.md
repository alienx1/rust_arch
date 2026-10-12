---
id: "019ca1ed-0000-7000-8003-000000000001"
code: "SYPG000"
system_code: "SY"
module_code: "SYMT"
route_path: "/"
icon: "DashboardOutlined"
title_th: "ภาพรวมระบบ"
title_en: "Dashboard"
parent_group: null
status: "Pending Scope"
tags:
  - program
  - system-core
  - dashboard
  - screen
---

# 📊 SYPG000 - ภาพรวมระบบ (Dashboard)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SYPG000`
> - **ชื่อภาษาไทย**: ภาพรวมระบบ
> - **Title (EN)**: Dashboard
> - **Route Path**: `/`
> - **Icon**: `DashboardOutlined` (`icondata_ai`)
> - **System**: `SY` (ระบบ / System)
> - **Module**: `SYMT` (ระบบงานหลัก / System Core)
> - **Parent Folder**: *(Root / ไม่มี Parent)*
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000001`
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอหลัก (Home / Landing Page) สำหรับผู้ใช้งานที่ผ่านการยืนยันตัวตนแล้ว แสดงข้อมูลสรุปภาพรวมการทำงาน สถานะระบบ สถิติการใช้งาน และ Quick Actions ต่างๆ ตามสิทธิ์ที่ผู้ใช้ได้รับ

---

## 📝 2. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)
<!-- 
  ส่วนนี้เปิดไว้สำหรับระบุ Scope ละเอียดของหน้าจอ:
  สามารถเขียน Requirement, Business Logic, หรือ Wireframe ที่ต้องการได้ที่นี่
-->

### 2.1 วัตถุประสงค์ (Objectives)
- *[ระบุวัตถุประสงค์ เช่น ให้ผู้บริหารดูสถิติภาพรวม, แจ้งเตือนงานค้าง]*

### 2.2 วิดเจ็ตและการแสดงผล (Widgets & Metrics)
- [ ] **KPI Summary Cards**:
  - จำนวนผู้ใช้งานทั้งหมด (Total Users)
  - จำนวนองค์กร/สาขา (Active Organizations / Branches)
  - จำนวน Audit Events วันนี้ (Today Audit Logs)
- [ ] **Quick Action Links**:
  - ทางลัดไปยังหน้าจอที่ใช้งานบ่อย เช่น [[DBPG001_ORGANIZATIONS|กำหนดองค์กร]], [[SUPG004_USER_ACCOUNTS|จัดการผู้ใช้]]
- [ ] **System Status / Health Feed**:
  - สถานะการเชื่อมต่อ Service (PostgreSQL, Valkey, Redpanda, Quickwit)

### 2.3 การกรองข้อมูลตามสิทธิ์ (ABAC / Data Scoping)
- ผู้ใช้มองเห็นข้อมูลเฉพาะใน Scope องค์กร/สาขา (`company_id`, `branch_id`, `department_id`) ที่ตนเองได้รับมอบหมาย

---

## 🔒 3. สิทธิ์การเข้าใช้งาน (Permissions & Security)
- API `/dashboard/...` อยู่หลัง Ory Kratos session และตรวจสิทธิ์ด้วย `routers/access.rs::require()` กับโปรแกรม `SYPG000` (ทุก `GET` ต้องมี `READ`)
- **สิทธิ์ที่กำหนดในระบบ (`su_permission`)**:
  - `READ`: เข้าชมหน้าจอ Dashboard และดูตัวชี้วัด (`GET /dashboard/summary`)
  - `WRITE`: ปรับแต่ง Layout หรือบันทึกการตั้งค่าวิดเจ็ตส่วนตัว
  - `APPROVE`: *(ถ้ามี)*
- **Authentication**: ต้องผ่านการ Login และมี Session ที่ถูกต้องจาก Ory Kratos

---

## 🗄️ 4. ตารางฐานข้อมูลที่เกี่ยวข้อง (Database Entities)
- ตารางสรุปข้อมูล:
  - `su_user` - ข้อมูลผู้ใช้งาน
  - `db_organization`, `db_branch` - โครงสร้างองค์กร
  - `su_configuration` - คอนฟิกค่าระบบ
- ตรวจสอบประวัติ:
  - Quickwit OpenSearch API (Querying audit log index)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**:
  - Component: `handler/web/application/master/src/screens/dashboard/`
  - UI Library: `shadcn/ui` (Cards, Charts, Metric badges)
- **Backend API (Axum 0.8)**:
  - Handler: `handler/api/master/src/handlers/dashboard.rs`
  - Route: `GET /api/v1/system/dashboard/summary`
- **Application Layer**:
  - Use Case: `application/master/src/usecases/dashboard/get_metrics.rs`
- **Domain Layer**:
  - Entity: `domain/src/entity/system/dashboard.rs`
