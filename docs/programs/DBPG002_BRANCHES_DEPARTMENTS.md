---
id: "019ca1ed-0000-7000-8003-000000000012"
code: "DBPG002"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg002"
icon: "BankOutlined"
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

# 🏛️ DBPG002 - Branches & Departments

> [!INFO] **Program Metadata**
> - **Program Code**: `DBPG002`
> - **Title (EN)**: Branches & Departments
> - **Route Path**: `/db/dbpg002`
> - **Icon**: `BankOutlined` (`icondata_ai`)
> - **System**: `DB` (Database)
> - **Module**: `DBMT` (Master Data)
> - **Parent Folder**: `DBPG000` (Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000012`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for managing organizational sub-units under a company — including branches (`db_branch`), regional offices, warehouses, and departments (`db_department`) — used for operational data scoping, cost centers, and employee access boundaries.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Define headquarters and sub-branches for each company, and manage departments under each branch.

### 2.2 Fields & Schema
- [x] **Branch Information (`db_branch`)**:
  - Branch Code: e.g. `HQ`, `B001`
  - Branch Name (TH / EN): `db_branch_label`
  - Parent Company: selected from `db_company`
  - Branch Type (Headquarters / Sub-branch): `branch_type` (`db_master_lookup`)
  - Address & Contacts: linked to `db_address_group` and `db_contact_group`
- [x] **Department Information (`db_department`)**:
  - Department Code: e.g. `IT`, `HR`, `ACC`
  - Department Name (TH / EN): `db_department_label`
  - Parent Branch: linked to `db_branch`

### 2.3 Business Rules & Actions
- [x] **Headquarters Rule**: Each company may have at most one headquarters branch (`branch_type = HEAD`).
- [x] **Default Permission Group of a department** (`db_department.default_permission_group_id`, optional): the accounts of the department's employees get that permission group **at the department's branch** (scope level `branch`), added automatically as an ordinary `su_user_assignment` row (`features/su/department_default_group.rs`):
  - when an account is created (SUPG004 adds it to what the admin picked; with nothing picked it is the account's only assignment, so its primary group);
  - when the default is set or changed on the department: the existing accounts of its employees that do not hold the group yet get it (in the same transaction as the department's save).
  - Nothing is ever taken away: changing or clearing the default leaves the rows already given (they are the account's own and are edited on SUPG004); an account that already holds the group at any scope is left as it is; an employee moving to another department keeps what they have.
  - Only an existing, active group can be chosen and **ADMIN cannot** (everyone in the department would become an administrator); a group switched off later is skipped for new accounts.
  - **Granting needs the right to assign groups**: setting or changing the default (create or update) needs **SUPG004 WRITE** as well as DBPG002 WRITE (403 otherwise), because it assigns a group to every account of the department, the editor's own included; leaving the default as it is, or clearing it, grants nothing and stays a DBPG002 matter. The group list for the selector (`GET /options/permission-groups`) needs READ on both programs. There is no further limit on which group (same as assigning on SUPG004; ADMIN is refused outright) and a manager can hold a group the editor does not have themselves.
  - Saving clears the ACL and menu caches (the accounts' permissions changed).
- [x] **Reference Check on Delete**: Branches or departments with dependent departments/employees cannot be deleted (`409 IN_USE`) and should be deactivated instead.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: View branch and department records
  - `WRITE`: Create and update branches and departments
  - `APPROVE`: Delete branches and departments
- **API Enforcement (program `DBPG002`)**: Every endpoint enforces `DBPG002` permissions — `READ` = view/search (`GET`), `WRITE` = create/update (`POST`/`PUT`), `APPROVE` = delete (`DELETE`); missing permission returns `403`.
- **Data Scoping**: Visibility is filtered by the user's `ViewScope`, and write/delete operations are checked against the user's assigned scope (`require_unit`: branches scoped by company, departments scoped by branch).

---

## 🗄️ 4. Database Entities
- `db_branch` (`id`, `company_id`, `code`, `branch_type_id`, `active`, ...)
- `db_branch_label` (`branch_id`, `language_id`, `label`)
- `db_department` (`id`, `branch_id`, `code`, `active`, ...)
- `db_department_label` (`department_id`, `language_id`, `label`)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg002/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg002/) (`Dbpg002Page`, `Dbpg002BranchPage`, `Dbpg002DepartmentPage`)
  - `/db/dbpg002`: Branch & Department list tabs with company filter, search, status filter, and `?company_id=` pre-selection
  - `/db/dbpg002/branch[/{id}]`: Branch form (`?company_id=`), `/db/dbpg002/department[/{id}]`: Department form (`?branch_id=`, `?company_id=`)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg002` ([`dbpg002_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg002_router.rs)):
  - `GET /companies`, `GET /summary`, `GET /branches/types`
  - `GET|POST /branches`, `GET|PUT /branches/{id}`, `GET|PUT /branches/{id}/contacts`
  - `GET|POST /departments`, `GET|PUT /departments/{id}`
  - `DELETE /branches/{id}`: permanently deletes the branch (along with its labels and address/contact groups in a single transaction) when unreferenced; headquarters branches cannot be deleted (`400`); if referenced by departments or employees, returns `409 IN_USE` (deactivate instead)
  - `DELETE /departments/{id}`: permanently deletes the department and its labels when unreferenced; if referenced, returns `409 IN_USE` (deactivate instead)
- **Application Layer**: [`features/db/dbpg002/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg002/) (`Dbpg002Queries`, `Dbpg002Commands`, `Dbpg002CommandExecutor`)
- **Enforced Business Rules**: Unique branch code per company; unique department code per branch; at most one headquarters (`branch_type = HEAD`) per company; optimistic locking via `row_version`; creating a branch automatically provisions its `db_address_group` and `db_contact_group` within the same transaction; departments inherit organization/company from their branch; a branch's parent company cannot be changed after creation.
- **Not Yet Implemented**: Branch street address editor UI, parent department hierarchy (no self-referencing column on `db_department`), active employee check before deactivation, and interactive tree view.
- **Domain Layer**: `domain::entity::{db_branch, db_branch_label, db_department, db_department_label}`
