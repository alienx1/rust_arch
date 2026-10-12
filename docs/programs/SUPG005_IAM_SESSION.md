---
id: "019ca1ed-0000-7000-8003-000000000025"
code: "SUPG005"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg005"
icon: "SafetyCertificateOutlined"
title_th: "สถานะเซสชัน (IAM)"
title_en: "IAM & Session Monitor"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - iam
  - session
  - ory-kratos
  - screen
---

# 🛡️ SUPG005 - สถานะเซสชัน (IAM & Session Monitor)

> [!INFO] **ข้อมูลโปรแกรม (Program Metadata)**
> - **Program Code**: `SUPG005`
> - **ชื่อภาษาไทย**: สถานะเซสชัน (IAM)
> - **Title (EN)**: IAM & Session Monitor
> - **Route Path**: `/su/supg005`
> - **API Base**: `/api/v1/su/supg005`
> - **Icon**: `SafetyCertificateOutlined`
> - **System**: `SU` | **Module**: `SUMT` | **Parent Folder**: `SUPG000`
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000025` (seed: `sql/52_program_catalogue.sql`)
> - **Backlink**: [[00_INDEX|กลับสู่สารบัญโปรแกรม (MOC)]]

---

## 🎯 ภาพรวมหน้าจอ
แสดงตัวตนและเซสชันของผู้ใช้ปัจจุบัน (Ory Kratos), ขอบเขตข้อมูล, สิทธิ์ตามโปรแกรม, อุปกรณ์ที่ล็อกอินอยู่ และตารางผู้ใช้งานทั้งระบบพร้อมการระงับ/เปิดใช้งานบัญชีและเพิกถอนเซสชัน

> [!NOTE] ความสัมพันธ์กับ SUPG004
> `SUPG004` (กำหนดผู้ใช้งาน) คือหน้าสร้าง/แก้ไขบัญชีและมอบหมายกลุ่มสิทธิ์ ส่วน `SUPG005` คือหน้าเฝ้าดูสถานะและสั่งงานเซสชันที่มีอยู่แล้ว (เดิมคือ `/iam/session`)

## 🔌 API (ต้องมี Kratos session, ล้มเหลวแบบ fail-closed 503 ถ้าไม่ได้ตั้งค่า Kratos)
| Method | Path | หน้าที่ |
|---|---|---|
| GET | `/api/v1/su/supg005/users?page=&limit=` | รายชื่อผู้ใช้พร้อมอุปกรณ์/เซสชัน |
| DELETE | `/api/v1/su/supg005/users/{id}/sessions` | เพิกถอนทุกเซสชันของผู้ใช้ |
| POST | `/api/v1/su/supg005/users/{id}/suspend` | ระงับบัญชี (ปิดที่ Kratos + เพิกถอนเซสชัน) |
| POST | `/api/v1/su/supg005/users/{id}/reactivate` | ยกเลิกการระงับ |
| DELETE | `/api/v1/su/supg005/sessions/{id}` | เพิกถอนเซสชันอุปกรณ์เดียว |

Kratos after-login hook `/api/v1/hooks/login-guard` ไม่ใช่โปรแกรม: เรียกโดย Kratos (ตรวจ `X-Webhook-Secret`) จึงคง path เดิมตามที่อ้างใน Kratos config ใช้ `Supg005Commands::authorize_login` ร่วมกัน

## 💻 Clean Architecture Implementation
- **Frontend**: [`pages/su/supg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/su/supg005/) (`Supg005Page`)
- **Router**: [`supg005_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/supg005_router.rs), hook: [`kratos_hook_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/kratos_hook_router.rs)
- **Application**: [`features/su/supg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/su/supg005/)
- **Tests**: `application/master/tests/supg005_test.rs`, `handler/api/master/tests/supg005_router_test.rs`
