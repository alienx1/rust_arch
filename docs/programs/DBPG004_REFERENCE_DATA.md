---
id: "019ca1ed-0000-7000-8003-000000000014"
code: "DBPG004"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg004"
icon: "GroupOutlined"
title_en: "Reference Data"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - reference-data
  - lookup
  - screen
---

# 📚 DBPG004 - Reference Data (Master Lookup)

> [!INFO] **Program Metadata**
> - **Program Code**: `DBPG004`
> - **Title (EN)**: Reference Data
> - **Route Path**: `/db/dbpg004`
> - **Icon**: `GroupOutlined` (`icondata_ai`)
> - **System**: `DB` (Database)
> - **Module**: `DBMT` (Master Data)
> - **Parent Folder**: `DBPG000` (Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000014`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for managing the platform's central reference lookup table (`db_master_lookup`) — such as name prefixes, company types, branch types, contact types, and system permission types — eliminating hardcoded constants in source code.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Allow administrators to manage lookup categories and localized dropdown items across the platform.

### 2.2 Fields & Schema
- [x] **Lookup Category**:
  - Category Code: e.g. `prefix`, `company_type`, `branch_type`, `contact_type`, `permission_type`
- [x] **Lookup Items (`db_master_lookup`)**:
  - Item Code: e.g. `HEAD`, `SUB`, `PHONE`, `TAX_ID`
  - Localized Labels: Thai and English (`db_master_lookup_label`)
  - Active Status: `active`
- [ ] **Currency Reference (`db_currency`)**:
  - ISO 4217 currency code (`THB`, `USD`, `EUR`), symbol (`฿`, `$`), decimal places (currently seeded via SQL)

### 2.3 Business Rules & Actions
- [x] **Cache Invalidation**: Updating master lookups automatically invalidates the `ref` and `org` namespaces in Valkey.
- [x] **System Protected Rows**: Core system lookup categories (such as `permission_type`: `READ`, `WRITE`, `APPROVE`) are protected against creation, deactivation, and deletion.

---

## 🔒 3. Permissions & Security
- All API routes sit behind Ory Kratos session authentication and enforce permissions via `routers/access.rs::require()` for program `DBPG004`: `READ` = view/search (`GET`), `WRITE` = create/update (`POST`/`PUT`), `APPROVE` = delete (`DELETE`).
- **Permissions defined in `su_permission`**:
  - `READ`: View and search lookup values (`GET` on all routes)
  - `WRITE`: Create or update lookup values (`POST` / `PUT`)
  - `APPROVE`: Delete lookup values (`DELETE`)

---

## 🗄️ 4. Database Entities
- `db_master_lookup` (`id`, `category`, `code`, `active`, ...)
- `db_master_lookup_label` (`master_lookup_id`, `language_id`, `label`)
- `db_currency`, `db_currency_label`
- `db_language`, `db_language_label`

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg004/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg004/) (`Dbpg004Page`, `Dbpg004LookupPage`)
  - `/db/dbpg004`: Lookup item list with category filter (showing item counts), code/label search, status filter, and 25-row pagination
  - `/db/dbpg004/lookup[/{id}]`: Lookup item form (`?category=` pre-selects a category; also allows entering a new category name)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg004` ([`dbpg004_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg004_router.rs)):
  - `GET /categories`, `GET /summary`
  - `GET|POST /lookups` (`?category=&search=&active=&page=&limit=`), `GET|PUT /lookups/{id}`
  - `DELETE /lookups/{id}` (`204 No Content`): permanently deletes the lookup item and its labels in a single transaction when unreferenced; if referenced by other tables, returns `409 IN_USE` (deactivate instead); items in system-protected categories (`permission_type`) cannot be deleted (`400`).
- **Application Layer**: [`features/db/dbpg004/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg004/) (`Dbpg004Queries`, `Dbpg004Commands`, `Dbpg004CommandExecutor`)
- **Enforced Business Rules**:
  - Unique `(category, code)` (case-insensitive code comparison); category must match `a-z 0-9 _`; code is normalized to uppercase.
  - Category and code are immutable after creation (since other tables and application logic reference them by code); only Thai/English labels and `active` status can be edited.
  - Category `permission_type` (`READ`/`WRITE`/`APPROVE`) is system-protected: new items cannot be added and existing items cannot be deactivated or deleted, though their display labels can be edited.
  - Thai label is required (English is optional: on create it copies the Thai label, on update blank values leave existing English labels untouched); enforces `row_version` optimistic locking and writes item + labels in a single transaction.
- **Not Yet Implemented**: Display sort order / default flag (no columns on `db_master_lookup`), currency management UI (`db_currency`), and active reference check before deactivation.
- **Domain Layer**: `domain::entity::{db_master_lookup, db_master_lookup_label}`
