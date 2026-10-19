---
id: "019ca1ed-0000-7000-8003-000000000001"
code: "SYPG000"
system_code: "SY"
module_code: "SYMT"
route_path: "/"
icon: "DashboardOutlined"
title_en: "Dashboard"
parent_group: null
status: "Pending Scope"
tags:
  - program
  - system-core
  - dashboard
  - screen
---

# 📊 SYPG000 - Dashboard

> [!INFO] **Program Metadata**
> - **Program Code**: `SYPG000`
> - **Title (EN)**: Dashboard
> - **Route Path**: `/`
> - **Icon**: `DashboardOutlined` (`icondata_ai`)
> - **System**: `SY` (System)
> - **Module**: `SYMT` (System Core)
> - **Parent Folder**: *(Root / No Parent)*
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000001`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Primary landing page (Home / Dashboard) for authenticated users, displaying platform KPI summaries, system status, usage statistics, and quick navigation shortcuts tailored to the user's permissions and View Scope.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Provide executives and operators with an at-a-glance overview of master data counts, security posture, and quick links to frequently used programs.

### 2.2 Widgets & Metrics
- [x] **KPI Summary Cards**:
  - Total User Accounts (`su_user`)
  - Active Organizations, Companies, Branches, and Departments
  - Total Employees and Reference Data counts
- [x] **Quick Action Links**:
  - Direct shortcuts to frequently used screens such as [[DBPG001_ORGANIZATIONS|Organizations & Companies]] and [[SUPG004_USER_ACCOUNTS|User Accounts]]
- [x] **System Status / Health Feed**:
  - Service connectivity and platform health indicators

### 2.3 ABAC / Data Scoping
- Users see organization/company/branch/department/employee counts filtered by their active `ViewScope` (`company_id`, `branch_id`, `department_id` — see [[12_VIEW_SCOPE_COMPANY_SWITCHER]]).

---

## 🔒 3. Permissions & Security
- The `/api/v1/dashboard/...` endpoints sit behind Ory Kratos session authentication and enforce permissions via `routers/access.rs::require()` for program `SYPG000` (all `GET` requests require `READ`).
- **Permissions defined in `su_permission`**:
  - `READ`: View the Dashboard screen and fetch summary metrics (`GET /api/v1/dashboard/summary`)
  - `WRITE`: Customize layout or personal widget preferences *(if applicable)*
  - `APPROVE`: *(N/A)*
- **Authentication**: Requires a valid Ory Kratos session.

---

## 🗄️ 4. Database Entities
- Summary count sources:
  - `su_user` — User accounts
  - `db_organization`, `db_company`, `db_branch`, `db_department`, `db_employee` — Organizational hierarchy
  - `su_configuration` — System configuration settings

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**:
  - Page: `handler/web/application/master/src/pages/dashboard/`
- **Backend API (Axum 0.8)**:
  - Router: `handler/api/master/src/routers/dashboard_router.rs`
  - Endpoint: `GET /api/v1/dashboard/summary`
- **Application Layer**:
  - Feature: `application/master/src/features/dashboard/`
