---
id: "019ca1ed-0000-7000-8003-000000000010"
code: "DBPG000"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg000"
icon: "ApartmentOutlined"
title_th: "หน้าหลักองค์กรและบริษัท"
title_en: "Organizations & Companies (Main)"
parent_group: null
status: "Implemented"
tags:
  - program
  - master-data
  - organization
  - company
  - main-screen
  - screen
---

# 🏢 DBPG000 - หน้าหลักองค์กรและบริษัท (Organizations & Companies)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `DBPG000`
> - **ชื่อภาษาไทย**: หน้าหลักองค์กรและบริษัท
> - **Title (EN)**: Organizations & Companies (Main Screen)
> - **Route Path**: `/db/dbpg000`
> - **Icon**: `ApartmentOutlined` (`icondata_ai`)
> - **System**: `DB` (ฐานข้อมูล / Database)
> - **Module**: `DBMT` (ข้อมูลหลัก / Master Data)
> - **Detail Sub-pages**:
>   - [[DBPG001_ORGANIZATIONS#หน้า-1-dbdbpg001org-จัดการองค์กร-org|/db/dbpg001/org (สร้าง / แก้ไข Org)]]
>   - [[DBPG001_ORGANIZATIONS#หน้า-2-dbdbpg001company-จัดการบริษัท-company|/db/dbpg001/company (สร้าง / แก้ไข Company)]]
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 1. ภาพรวมหน้าจอ (Screen Overview)
หน้าจอศูนย์กลางสำหรับค้นหาและบริหารจัดการโครงสร้างองค์กรและบริษัท (Organizations & Companies Dashboard) ประกอบด้วย:
1. **Dropdown สำหรับเลือกองค์กร (Select Org)** เพื่อกรองดูข้อมูลเฉพาะกลุ่มบริษัทในเครือ
2. **Search Bar สำหรับค้นหาบริษัท (Search Company)** ค้นหาแบบ Real-time
3. **Table แสดงรายชื่อบริษัท (Company Table)** แสดงสถานะ, สกุลเงิน, เลขผู้เสียภาษี
4. **Action Buttons** สำหรับเปิดไปยังหน้า Detail 2 หน้า:
   - `/db/dbpg001/org` สำหรับ Create / Update Organization
   - `/db/dbpg001/company` สำหรับ Create / Update Company

---

## 🖼️ 2. UI Mockup & Wireframe

### 2.1 Modern Teal Theme UI Mockup (shadcn/ui OKLCH)
![UI Mockup - DBPG000 Organizations & Companies (Teal Theme)](./assets/dbpg000_teal_theme.jpg)

### 2.2 Neutral Theme Mockup
![UI Mockup - DBPG000 Organizations & Companies (Neutral Theme)](./assets/dbpg000_ui_mockup.jpg)

### 2.3 ASCII Layout Wireframe
```
+----------------------------------------------------------------------------------------------------+
|  🏢 จัดการองค์กรและบริษัท (Organizations & Companies)                             [+ เพิ่ม Org] [+ เพิ่ม Company] |
+----------------------------------------------------------------------------------------------------+
|  [ 🏢 เลือกองค์กร: [ บริษัท แม่ เอบีซี โฮลดิ้ง จำกัด (ORG001)  ▼ ] ] [ ✏️ แก้ไข Org ]                      |
|                                                                                                    |
|  🔍 ค้นหาบริษัท: [ พิมพ์ชื่อบริษัท, รหัสบริษัท, หรือ Tax ID...                                  🔍 ] |
+----------------------------------------------------------------------------------------------------+
|  ตารางรายชื่อบริษัท (Companies)                                                         พบ 3 บริษัท  |
+------------+---------------------------+-----------------------+-------------+----------+---------+--------+
| รหัสบริษัท | ชื่อบริษัท (ไทย)            | ชื่อบริษัท (EN)        | Tax ID      | สกุลเงิน | สถานะ   | จัดการ |
+------------+---------------------------+-----------------------+-------------+----------+---------+--------+
| CMP001     | บจก. เอบีซี เทรดดิ้ง      | ABC Trading Co., Ltd. | 01055500001 | THB (฿)  | Active  | [✏️] [👁️] |
| CMP002     | บจก. เอบีซี โลจิสติกส์    | ABC Logistics Ltd.    | 01055500002 | THB (฿)  | Active  | [✏️] [👁️] |
| CMP003     | บจก. เอบีซี ดิจิทัล โซลูชั่น| ABC Digital Solutions | 01055500003 | THB (฿)  | Inactive| [✏️] [👁️] |
+------------+---------------------------+-----------------------+-------------+----------+---------+--------+
|  [ ◀ หน้า 1 จาก 1 ▶ ]                                                      แสดงแถวละ [ 10 ▼ ] รายการ |
+----------------------------------------------------------------------------------------------------+
```

---

## 📝 3. ขอบเขตและข้อกำหนดการทำงาน (Scope & Requirements)

### 3.1 องค์ประกอบบนหน้าจอ (Screen Components)
1. **Header Actions**:
   - `[+ เพิ่มองค์กร]` -> นำทางไปยัง Route `/db/dbpg001/org` (โหมด Create)
   - `[+ เพิ่มบริษัท]` -> นำทางไปยัง Route `/db/dbpg001/company` (โหมด Create พร้อม Default `org_id` ตามที่เลือกใน Dropdown)

2. **Org Selector Dropdown**:
   - ดึงรายการจากตาราง `db_organization` + `db_organization_label` (กรองเฉพาะ active = true)
   - แสดงรูปแบบ: `<ชื่อองค์กร> (<รหัสองค์กร>)`
   - ตัวเลือกแรก: `ทั้งหมด (All Organizations)` สำหรับดูภาพรวมบริษัททุกเครือ
   - ปุ่ม `[✏️ แก้ไข Org]` ข้าง Dropdown: เมื่อคลิกจะพาไปยัง `/db/dbpg001/org?id={selected_org_id}` เพื่อแก้ไขข้อมูลองค์กรที่เลือกทันที

3. **Company Search Bar**:
   - ค้นหาแบบ Debounced Typeahead (300ms)
   - รองรับการค้นหาข้ามภาษา (Partial Match):
     - รหัสบริษัท (`db_company.code`)
     - ชื่อบริษัทภาษาไทย (`db_company_label.label` TH)
     - ชื่อบริษัทภาษาอังกฤษ (`db_company_label.label` EN)
     - เลขประจำตัวผู้เสียภาษี (Tax ID / Registration Number)

4. **Company Data Table**:
   - **คอลัมน์ในตาราง**:
     - `รหัสบริษัท` (Code)
     - `ชื่อบริษัท (ไทย)`
     - `ชื่อบริษัท (EN)`
     - `องค์กรแม่` (แสดงเฉพาะเมื่อเลือกดู All Organizations)
     - `Tax ID`
     - `สกุลเงินหลัก` (Currency Code เช่น THB, USD)
     - `สถานะ` (Active / Inactive Badge)
     - `การกระทำ (Actions)`:
       - ✏️ **Edit**: ลิงก์ไปยัง `/db/dbpg001/company?id={company_id}`
       - 👁️ **Branches**: ทางลัดไปดูสาขาของบริษัทนี้ใน [[DBPG002_BRANCHES_DEPARTMENTS|DBPG002]] (`/db/dbpg002?company_id={company_id}`)

### 3.2 กฎเกณฑ์และเงื่อนไขทางธุรกิจ (Business Rules)
- **Data Permission**: ผู้ใช้งานจะเห็นเฉพาะ Organization และ Company ที่ได้รับสิทธิ์ตาม ABAC Scope (`su_user.company_id` หรือ Group Scopes)
- **Pagination & Sorting**: รองรับ Pagination (10, 25, 50 รายการ) และคลิกเรียงลำดับ (Sort Asc/Desc) ตามรหัสบริษัท หรือชื่อบริษัท

---

## 🔗 4. หน้า Detail เชื่อมโยง (Sub-Pages)
- 📄 **[[DBPG001_ORGANIZATIONS#หน้า-1-dbdbpg001org-จัดการองค์กร-org|/db/dbpg001/org]]**: หน้าจอ Create / Update ข้อมูลองค์กร
- 📄 **[[DBPG001_ORGANIZATIONS#หน้า-2-dbdbpg001company-จัดการบริษัท-company|/db/dbpg001/company]]**: หน้าจอ Create / Update ข้อมูลบริษัท

---

## 🔒 5. สิทธิ์การเข้าใช้งาน (Permissions & Security)
ระบบตรวจสิทธิ์ของโปรแกรม `DBPG000` ที่ฝั่ง API ทุก endpoint (ต้องล็อกอินผ่าน Kratos ก่อน):
- **`READ`** = ดู/ค้นหา: ทุก GET (รายชื่อองค์กร/บริษัท, รายละเอียด, ประเภทบริษัท, สรุป KPI) — หน้านี้เข้าจากเมนู `DBPG001` (ข้อมูลองค์กร) จึงยอมรับ READ ของ `DBPG000` **หรือ** `DBPG001` อย่างใดอย่างหนึ่ง
- **`WRITE`** = เพิ่ม/แก้ไข: โปรแกรมนี้เป็นหน้าอ่านอย่างเดียว การเพิ่ม/แก้ไขทำผ่าน `DBPG001`
- **`APPROVE`** = ลบ: ไม่มี endpoint ลบในโปรแกรมนี้ (การลบอยู่ที่ `DBPG001`)
- ไม่มีสิทธิ์ => HTTP 403 `FORBIDDEN`

---

## 💻 6. Clean Architecture Implementation
- **Frontend SPA (Leptos 0.8 CSR WebAssembly)**:
  - Route: `/organization` (และ alias `/db/dbpg000`)
  - Page Entry: [`handler/web/application/master/src/pages/db/dbpg000/mod.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/mod.rs) (`Dbpg000Page`)
  - Screen Orchestrator: [`handler/web/application/master/src/pages/db/dbpg000/screens/dbpg000_screen.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/screens/dbpg000_screen.rs)
  - Components:
    - [`org_stats_cards.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/components/org_stats_cards.rs): สรุป KPI ยอดรวมกลุ่มองค์กร และบริษัทที่เปิด/ปิดใช้งาน
    - [`org_selector_bar.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/components/org_selector_bar.rs): Dropdown เลือกกลุ่มองค์กร + ปุ่ม Add Org / Edit Org / Add Company
    - [`company_search_filter.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/components/company_search_filter.rs): ค้นหา Real-time หลายมิติ + ปุ่มกรองสถานะ Active/Inactive
    - [`company_data_table.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/components/company_data_table.rs): ตารางข้อมูลบริษัท, รหัส, ชื่อไทย/อังกฤษ, สังกัดองค์กร, ประเภท, สถานะ, ปุ่มแก้ไขและทางลัดสาขา
  - Provider & Context: [`providers/dbpg000_context.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg000/providers/dbpg000_context.rs) (จัดการ Reactive Error Signal)
  - Empty State UI: แสดงผลหน้าจอพร้อมตารางเปล่า (`Empty Table State`: "ไม่พบข้อมูลบริษัท" พร้อมปุ่ม "+ เพิ่มบริษัทใหม่") และแสดงแถบเตือนสถานะเมื่อยังไม่สามารถเชื่อมต่อฐานข้อมูลได้ แทนการตัดเข้าหน้า 503 เต็มจอ
- **Backend API Delivery Tier (Axum 0.8)**:
  - Router: [`handler/api/master/src/routers/dbpg000_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg000_router.rs)
  - Endpoints:
    - `GET /api/v1/db/dbpg000/organizations` (ดึงรายการองค์กรพร้อมชื่อภาษาและจำนวนบริษัท)
    - `GET /api/v1/db/dbpg000/organizations/{id}` (ดึงองค์กรรายตัว)
    - `GET /api/v1/db/dbpg000/organizations/summary` (ดึงสรุป KPI สถิติภาพรวม)
    - `GET /api/v1/db/dbpg000/companies` (ดึงรายการบริษัท กรองตามองค์กร, ค้นหา, สถานะ, แบ่งหน้า)
    - `GET /api/v1/db/dbpg000/companies/{id}` (ดึงบริษัทรายตัว)
    - `GET /api/v1/db/dbpg000/companies/types` (ดึงประเภทนิติบุคคลจาก db_master_lookup)
- **Application CQRS Layer (`application-master`)**:
  - Feature: [`application/master/src/features/db/dbpg000/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg000/)
  - Trait Commands: `OrganizationCommands` & Executor: `OrganizationCommandExecutor`
  - Trait Queries: `OrganizationQueries` & Runner: `DbOrganizationQueries` (Sea-ORM 2.0)
  - Models & DTOs: `models/mod.rs`
  - Errors: `errors/mod.rs` (`OrganizationError`)
- **Domain Entities Layer (`domain`)**:
  - `domain::entity::db_organization`
  - `domain::entity::db_organization_label`
  - `domain::entity::db_company`
  - `domain::entity::db_company_label`
  - `domain::entity::db_master_lookup`
  - `domain::entity::db_master_lookup_label`
