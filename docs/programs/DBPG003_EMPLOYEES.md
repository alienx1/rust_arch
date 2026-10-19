---
id: "019ca1ed-0000-7000-8003-000000000013"
code: "DBPG003"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg003"
icon: "IdcardOutlined"
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

# 👤 DBPG003 - Employees

> [!INFO] **Program Metadata**
> - **Program Code**: `DBPG003`
> - **Title (EN)**: Employees
> - **Route Path**: `/db/dbpg003`
> - **Icon**: `IdcardOutlined` (`icondata_ai`)
> - **System**: `DB` (Database)
> - **Module**: `DBMT` (Master Data)
> - **Parent Folder**: `DBPG000` (Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000013`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for managing employee records (`db_employee`), multilingual person names (`db_person_name`, `db_person_name_label`), branch/department assignments, personal contact details, profile photos, and integration with platform user accounts (`su_user`).

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Maintain employee records, assign company/branch/department affiliations, manage contact details, and synchronize active status with linked login accounts.

### 2.2 Fields & Schema
- [x] **Personal Information**:
  - Employee Code: `code`
  - Title Prefix / First Name / Last Name / Nickname (TH / EN): `db_person_name`, `db_person_name_label`
  - Contacts & National ID Card (`ID_CARD`): stored via `db_contact_group` / `db_contact`
- [x] **Organizational Affiliation**:
  - Parent Company (`db_company`)
  - Branch (`db_branch`)
  - Department (`db_department`)
- [x] **User Account Linkage**:
  - Linked with `su_user.employee_id` for employees granted ERP system access

### 2.3 Business Rules & Actions
- [x] **Employee Search & Filter**: Filter by company, branch, department, active status, or search by code, first name, last name, or nickname.
- [x] **Deactivation & Account Suspension**: Deactivating an employee automatically suspends all linked `su_user` accounts and revokes active Ory Kratos sessions.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: View employees within the user's permitted branch/department scope
  - `WRITE`: Create and update employee records
  - `APPROVE`: Delete employee records
- **API Enforcement (program `DBPG003`)**: Every endpoint enforces `DBPG003` permissions — `READ` = view/search (`GET`), `WRITE` = create/update (`POST`/`PUT`), `APPROVE` = delete (`DELETE`); missing permission returns `403`.
- **Data Scoping**: List queries are filtered by `ViewScope`, and write/delete operations require department-level scope coverage (`require_in`; transferring an employee across departments requires permission on both the old and new departments).

---

## 🗄️ 4. Database Entities
- `db_employee` (`id`, `company_id`, `branch_id`, `department_id`, `person_name_id`, `code`, `image_key`, `active`, ...)
- `db_person_name` (`id`, `active`)
- `db_person_name_label` (`person_name_id`, `language_id`, `prefix`, `first_name`, `last_name`, `nick_name`, ...)
- `db_contact_group`, `db_contact`
- `su_user` (linked via `employee_id`)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg003/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg003/) (`Dbpg003Page`, `Dbpg003EmployeePage`)
  - `/db/dbpg003`: Employee list with company/branch filters, search by code/first name/last name/nickname, status filter, and 25-row pagination
  - `/db/dbpg003/employee[/{id}]`: Employee form (`?company_id=` pre-selects company) with cascading Company → Branch → Department selectors
- **Deactivating an Employee = Suspending Linked Accounts**: Saving an employee as `inactive` immediately suspends all `su_user` accounts linked to that employee (disables Kratos identity + revokes active sessions + sets `active=false`). If suspension fails, an error is returned and saving can be retried; the Kratos `login-guard` hook also rejects sign-ins for inactive employees. Reactivating an employee (`inactive` → `active`) automatically reactivates linked accounts (only triggered on state transition, so manual account suspensions on already-active employees are preserved).
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg003` ([`dbpg003_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg003_router.rs)):
  - `GET /companies`, `/branches`, `/departments`, `/prefixes`, `/summary`
  - `GET|POST /employees` (`?page=&limit=&search=&active=&company_id=&branch_id=&department_id=`), `GET|PUT /employees/{id}`, `GET|PUT /employees/{id}/contacts`
  - **Employee Photo** (`db_employee.image_key`, stored in Garage S3 at `images/employees/{id}/<uuidv7>.<ext>`): `GET|PUT|DELETE /employees/{id}/image` — `PUT` accepts raw image bytes and returns a DTO with `image_url`; `GET` requires `READ`, `PUT`/`DELETE` require `WRITE` scoped to the employee's department. File validation rules (PNG/JPEG/GIF/WebP ≤ 2 MB verified by magic bytes; SVG rejected) and behavior match org/company logos in [[DBPG001_ORGANIZATIONS]], backed by `application/master/src/features/db/image_object_store.rs` and `web_package::ui::component::ImageField`.
  - `DELETE /employees/{id}` (`204 No Content`): permanently deletes `db_person_name` + labels and the `db_employee` row in a single transaction when unreferenced (if a linked `su_user` account exists, delete it in `SUPG004` first); if referenced, returns `409 IN_USE` (deactivate instead).
- **Application Layer**: [`features/db/dbpg003/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg003/) (`Dbpg003Queries`, `Dbpg003Commands`, `Dbpg003CommandExecutor`)
- **Enforced Business Rules**: Unique employee code within a department; Thai first/last name required (English is optional: on create it defaults to the Thai name, on update blank values leave existing English labels untouched); title prefix must exist in lookup category `prefix`; organization/company/branch are derived from the selected department; optimistic locking via `row_version`; `db_person_name` + labels + `db_employee` are written in a single transaction.
- **Query Execution**: Filtering, search, and pagination execute in SQL, loading localized labels only for the rows on the current page.
- **Not Yet Implemented**: Date of birth / hire date / job title columns (not in `db_employee` schema), street address editor (`db_address_group`), and automatic employee code generator.
- **Domain Layer**: `domain::entity::{db_employee, db_person_name, db_person_name_label}`
