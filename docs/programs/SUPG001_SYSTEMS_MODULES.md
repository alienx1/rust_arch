---
id: "019ca1ed-0000-7000-8003-000000000021"
code: "SUPG001"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg001"
icon: "ClusterOutlined"
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

# 🧩 SUPG001 - Systems & Modules

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG001`
> - **Title (EN)**: Systems & Modules
> - **Route Path**: `/su/supg001`
> - **Icon**: `ClusterOutlined` (`icondata_ai`)
> - **System**: `SU` (Security)
> - **Module**: `SUMT` (Security Maintenance)
> - **Parent Folder**: `SUPG000` (Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000021`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for system administrators to register and maintain top-level application structure — specifically sub-systems (`db_system`) and functional modules (`db_module`) — which organize sidebar navigation and permission matrix groupings.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Register new application systems (e.g. Accounting, Inventory) and sub-modules, and manage their active status and localized labels.

### 2.2 Fields & Schema
- [x] **System (`db_system`)**:
  - System Code: e.g. `SY`, `DB`, `SU`, `AL` (2–10 uppercase alphanumeric characters)
  - Localized Names (TH / EN): `db_system_label`
  - Active Status: `active`
- [x] **Module (`db_module`)**:
  - Module Code: e.g. `SYMT`, `DBMT`, `SUMT`, `ALMT`
  - Parent System: references `db_system.id`
  - Localized Names (TH / EN): `db_module_label`
  - Active Status: `active`

### 2.3 Business Rules & Actions
- [x] **Code Immutability**: Once created, system and module codes cannot be modified because they are referenced in permission definitions and routing conventions.
- [x] **Lock-Out Prevention**: Core security system `SU` and module `SUMT` cannot be deactivated or deleted.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: View systems and modules
  - `WRITE`: Create or update systems and modules
  - `APPROVE`: Delete systems or modules (`DELETE` requires `APPROVE`)
- **Restricted Access**: Intended for the `ADMIN` permission group.

---

## 🗄️ 4. Database Entities
- `db_system` (`id`, `code`, `active`, ...)
- `db_system_label` (`system_id`, `language_id`, `label`)
- `db_module` (`id`, `system_id`, `code`, `active`, ...)
- `db_module_label` (`module_id`, `language_id`, `label`)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `handler/web/application/master/src/pages/su/supg001/` (System / Module tabs, `UnitForm`) • routes `/su/supg001`, `/su/supg001/{system|module}[/{id}]`
- **Backend API** (`/api/v1/su/supg001`, requires Kratos session; `READ` = `GET`, `WRITE` = `POST`/`PUT`, `APPROVE` = `DELETE`): `GET|POST /systems`, `PUT|DELETE /systems/{id}`, `GET|POST /modules`, `PUT|DELETE /modules/{id}`
- **Application**: `application/master/src/features/su/supg001/` (validates 2–10 uppercase `A-Z0-9` codes; codes are immutable after creation; enforces optimistic locking via `row_version`)
- **Deletion**: `DELETE /systems/{id}` and `DELETE /modules/{id}` (`204 No Content`) permanently delete the record and its labels in a single transaction; `SU`/`SUMT` cannot be deleted (`400`); if referenced by child modules, programs, or permissions, returns `409 IN_USE` (deactivate instead)
- **Key Guardrail**: System `SU` and module `SUMT` cannot be deactivated (prevents administrative lock-out)
- **Tests**: `application/master/tests/supg001_test.rs`, `handler/api/master/tests/supg001_router_test.rs`
