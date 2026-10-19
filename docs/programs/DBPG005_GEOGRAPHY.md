---
id: "019ca1ed-0000-7000-8003-000000000015"
code: "DBPG005"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg005"
icon: "GlobalOutlined"
title_en: "Geography"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - geography
  - province
  - district
  - screen
---

# 🌍 DBPG005 - Geography (Provinces / Districts / Sub-Districts / Postal Codes)

> [!INFO] **Program Metadata**
> - **Program Code**: `DBPG005`
> - **Title (EN)**: Geography
> - **Route Path**: `/db/dbpg005`
> - **Icon**: `GlobalOutlined` (`icondata_ai`)
> - **System**: `DB` (Database)
> - **Module**: `DBMT` (Master Data)
> - **Parent Folder**: `DBPG000` (Master Data)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000015`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for searching and maintaining geographical master data — including countries (`db_country`), provinces (`db_province`), districts (`db_district`), sub-districts (`db_sub_district`), and postal codes (`db_postal_code`) — used across organization, company, branch, and employee addresses.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Maintain administrative division hierarchies and postal codes with fast cascading filtering.

### 2.2 Fields & Schema
- [x] **Geographical Hierarchy**:
  - Country: ISO code, e.g. `THA`
  - Province: province code, Thai / English names
  - District: linked to `province_id`
  - Sub-District: linked to `district_id`
  - Postal Code: 5-digit code linked to province, district, and sub-district
- [ ] **Thai Collation**:
  - PostgreSQL ICU Collation `th-TH-x-icu` for dictionary-order Thai sorting

### 2.3 Business Rules & Actions
- [x] **Cascading Dropdowns**: Selecting a province filters districts within that province -> selecting a district filters sub-districts -> filters postal codes.
- [ ] **Postal Code Quick Search**: Auto-filling sub-district, district, and province from a postal code in address forms.

---

## 🔒 3. Permissions & Security
- All API routes sit behind Ory Kratos session authentication and enforce permissions via `routers/access.rs::require()` for program `DBPG005`: `READ` = view/search (`GET`), `WRITE` = create/update (`POST`/`PUT`), `APPROVE` = delete (`DELETE`).
- **Permissions defined in `su_permission`**:
  - `READ`: Search and view geographical records (`GET` on all routes)
  - `WRITE`: Create or update provinces, districts, sub-districts, and postal codes (`POST` / `PUT`)
  - `APPROVE`: Delete geographical records (`DELETE`)

---

## 🗄️ 4. Database Entities
- `db_country`, `db_country_label`
- `db_province`, `db_province_label`
- `db_district`, `db_district_label`
- `db_sub_district`, `db_sub_district_label`
- `db_postal_code`

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg005/) (`Dbpg005Page`, `Dbpg005FormPage`)
  - `/db/dbpg005`: Tabs for Provinces / Districts / Sub-Districts / Postal Codes with cascading Province → District filters, search, status filter, and 25-row server-side pagination
  - `/db/dbpg005/{province|district|sub-district|postal-code}[/{id}]`: Create/Edit forms (`?province_id=`, `?district_id=` pre-select parent area) with cascading Province → District → Sub-District selectors
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg005` ([`dbpg005_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg005_router.rs)):
  - `GET /summary`
  - `GET|POST /provinces`, `GET|PUT /provinces/{id}`
  - `GET|POST /districts` (`?province_id=`), `GET|PUT /districts/{id}`
  - `GET|POST /sub-districts` (`?province_id=&district_id=`), `GET|PUT /sub-districts/{id}`
  - `GET|POST /postal-codes` (`?province_id=&district_id=&sub_district_id=&search=`), `GET|PUT /postal-codes/{id}`
  - `DELETE /provinces/{id}` — permanently deletes the province and its labels in a single transaction if unreferenced; if referenced by districts, postal codes, or addresses, returns `409 IN_USE` (deactivate instead)
  - `DELETE /districts/{id}` — permanently deletes the district and its labels in a single transaction if unreferenced; if referenced by sub-districts, postal codes, or addresses, returns `409 IN_USE` (deactivate instead)
  - `DELETE /sub-districts/{id}` — permanently deletes the sub-district and its labels in a single transaction if unreferenced; if referenced by postal codes or addresses, returns `409 IN_USE` (deactivate instead)
  - `DELETE /postal-codes/{id}` — permanently deletes the postal code if unreferenced by addresses; if referenced, returns `409 IN_USE` (deactivate instead)
  - All list endpoints support `search`, `active`, `page`, and `limit` (up to 200 per page, sufficient to load all 77 provinces in a single dropdown call)
- **Application Layer**: [`features/db/dbpg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg005/) (`Dbpg005Queries`, `Dbpg005Commands`, `Dbpg005CommandExecutor`)
- **Enforced Business Rules**:
  - Unique codes within parent scope: province code within country, district code within province, sub-district code within district, and postal code within the same sub-district/district/province combination.
  - Codes and parent area links are immutable after creation (referenced by child divisions and addresses); only labels and `active` status can be updated (postal code digits can be edited).
  - Ancestor IDs for districts, sub-districts, and postal codes are derived from the most specific selected parent division to prevent inconsistent hierarchies; new provinces automatically bind to the active country.
  - Thai name is required (English is optional: on create it copies the Thai name, on update blank values leave existing English labels untouched); enforces `row_version` optimistic locking and writes entity + labels in a single transaction.
- **Not Yet Implemented**: Country management UI (`db_country`; seeded with `THA`), postal code auto-fill widget inside address forms, `th-TH-x-icu` sorting (currently sorted by code), and active address reference check before deactivation.
- **Domain Layer**: `domain::entity::{db_province, db_district, db_sub_district, db_postal_code}` and their corresponding `*_label` entities.
