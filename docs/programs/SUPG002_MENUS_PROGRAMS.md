---
id: "019ca1ed-0000-7000-8003-000000000022"
code: "SUPG002"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg002"
icon: "MenuOutlined"
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

# 📑 SUPG002 - Menus & Programs

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG002`
> - **Title (EN)**: Menus & Programs
> - **Route Path**: `/su/supg002`
> - **Icon**: `MenuOutlined` (`icondata_ai`)
> - **System**: `SU` (Security)
> - **Module**: `SUMT` (Security Maintenance)
> - **Parent Folder**: `SUPG000` (Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000022`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for managing the sidebar navigation hierarchy and registering platform programs (`db_program`) — including parent folders and leaf screens, SPA route paths, icons (`icondata_ai`), display sort order, and permission-filtered menu tree construction (`get_menu_tree`).

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Maintain the program catalogue, organize parent-child menu folders, configure route paths and icons, and adjust menu sort order.

### 2.2 Fields & Schema
- [x] **Program & Menu Details (`db_program`)**:
  - Program Code: e.g. `DBPG001`, `SUPG001`
  - Node Type: Folder (`path` is `NULL`) or Leaf Screen (`path` specifies the SPA route)
  - Parent System (`system_id`) and Module (`module_id`)
  - Parent Folder (`program_id` self-referencing FK)
  - Localized Menu Labels: Thai and English (`db_program_label`)
  - Icon Name: e.g. `DashboardOutlined`, `ApartmentOutlined`
  - Sort Order: integer (`sort_order`)
  - Active Status: `active` (`boolean`)

### 2.3 Business Rules & Actions
- [x] **Permission-Filtered Menu Tree**: When the frontend calls `GET /api/v1/su/supg002/menu`, the returned tree includes only programs on which the user holds `READ` permission.
- [x] **Ancestor Folder Inclusion**: When a user has access to a child program, its active ancestor folders are automatically included in the menu tree; deactivating a folder hides all items beneath it.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: View the program catalogue and menu structure
  - `WRITE`: Create, update, and reorder menus and programs
  - `APPROVE`: Delete programs (`DELETE` requires `APPROVE`)

---

## 🗄️ 4. Database Entities
- `db_program` (`id`, `system_id`, `module_id`, `program_id`, `code`, `path`, `icon`, `sort_order`, `active`, ...)
- `db_program_label` (`program_id`, `language_id`, `label`)
- `su_permission` (joined to filter accessible menu items)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg002/` (server-paged program table + system/module/status filters, `ProgramForm`) • routes `/su/supg002`, `/su/supg002/program[/{id}]`
- **Backend API** (`/api/v1/su/supg002`): `GET /menu` (per-user sidebar menu tree), `GET /options`, `GET|POST /programs`, `GET|PUT|DELETE /programs/{id}`
- **Application**: `features/su/supg002/` (`build_menu_tree`: inactive folders hide their entire subtree; `path` is normalized to lowercase and prefixed with `/`; validates that parent folders belong to the same system and contain no cycles; enforces optimistic locking via `row_version`)
- **Deletion**: `DELETE /programs/{id}` (`204 No Content`) permanently deletes the program along with its labels and `su_permission` grants in a single transaction; core security screens `SUPG001`/`SUPG002`/`SUPG003` cannot be deleted (`400`); if child programs still reference the folder, returns `409 IN_USE` (deactivate instead)
- **Key Guardrails**: `SUPG001`, `SUPG002`, and `SUPG003` cannot be deactivated or hidden; system, module, and program code are immutable after creation
- **Tests**: `supg002_test.rs`, `supg002_router_test.rs`
