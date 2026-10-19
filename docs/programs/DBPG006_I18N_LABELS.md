---
id: "019ca1ee-0000-7000-8003-000000000001"
code: "DBPG006"
system_code: "DB"
module_code: "DBMT"
route_path: "/db/dbpg006"
icon: "TranslationOutlined"
title_en: "i18n Labels"
parent_group: "DBPG000"
status: "Implemented"
tags:
  - program
  - master-data
  - i18n
  - label
  - screen
---

# 🌐 DBPG006 - i18n Labels

> [!INFO] **Program Metadata**
> - **Program Code**: `DBPG006`
> - **Title (EN)**: i18n Labels
> - **Route Path**: `/db/dbpg006`
> - **Icon**: `TranslationOutlined` (`icondata_ai`)
> - **System**: `DB` (Database)
> - **Module**: `DBMT` (Master Data)
> - **Parent Folder**: `DBPG000` (Master Data)
> - **Database UUID**: `019ca1ee-0000-7000-8003-000000000001`
> - **Seed**: [`sql/52_program_catalogue.sql`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/sql/52_program_catalogue.sql)
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for system administrators to manage the platform's UI translation dictionary (`db_label`), which is served to clients via `GET /api/v1/i18n/...`. Administrators can search, add, edit, and delete translation entries across all active languages without modifying SQL files.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Allow administrators to view and edit translations across all supported languages (currently `th` and `en`) for every category in a single unified view.
- One **logical entry** corresponds to a unique `(category, key)` pair, backed by one row per language in `db_label`.

### 2.2 Fields & Schema
- **Category (`category`)**: 1–100 characters, restricted to `A-Z a-z 0-9 _ . -` (trimmed), e.g. `menu`, `error`, `supg003`.
- **Key (`key`)**: non-empty, up to 300 characters (trimmed), e.g. `column.thai_name`.
- **Value (`value`)**: per language, up to 2,000 characters (trimmed); an empty input removes the translation row for that specific language.
- The form dynamically renders one input field per active language along with the language's localized name (`db_language_label`).

### 2.3 Business Rules & Actions
- Duplicate `(category, key)` entries cannot be created (returns `409 DUPLICATE`).
- Saving (create or update) writes all languages in a single database transaction: non-empty values insert a new row (UUIDv7, `row_version = 1`) or update the existing row (`row_version + 1`); empty values delete the row for that language.
- **Optimistic locking**: Because a single `(category, key)` entry spans multiple `db_label` rows (one per language), `row_version` is computed as **the sum of `row_version` across all language rows for that entry** (returned on `LabelEntryDto` and sent back in `UpdateLabelCommand.row_version`). On save, the transaction locks the entry's rows and compares the sum; a mismatch returns `412 VERSION_CONFLICT`.
- At least one language must have a non-empty value, and every `language_id` must belong to an active language.
- Category and key are immutable after creation (since code references labels by `<category>.<key>`); to rename a key, delete the entry and create a new one.
- Deleting an entry deletes all `db_label` rows for that `(category, key)` (no other tables hold foreign keys to `db_label`).
- **Cache**: Writing to `DBPG006` invalidates the `i18n` and `dashboard` namespaces in Valkey and immediately triggers a background refill of the `th` and `en` dictionaries (see [[11_VALKEY_READ_CACHE]]).

---

## 🔒 3. Permissions & Security
- All API routes sit behind Ory Kratos session authentication and enforce permissions via `routers/access.rs::require()` for program `DBPG006`.
- **Permissions defined in `su_permission`** (`ADMIN` group holds all three):
  - `READ`: List, search, and view label entries (`GET`)
  - `WRITE`: Create and update label entries (`POST` / `PUT`)
  - `APPROVE`: Delete label entries (`DELETE`)

---

## 🗄️ 4. Database Entities
- `db_label` (`id`, `language_id`, `category`, `key`, `value`, `change_by`, `row_version`) — unique on `(category, key, language_id)`
- `db_language`, `db_language_label` (active languages and display names)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos WebAssembly)**: [`pages/db/dbpg006/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/db/dbpg006/) (`Dbpg006Page`, `Dbpg006LabelPage`)
  - `/db/dbpg006`: Label entry list with category filter, search across category/key/value, side-by-side values for all languages, and 25-row pagination
  - `/db/dbpg006/label[?category=&key=]`: Label entry form (pass `category` + `key` to edit, or `?category=` to pre-select a category when creating; supports typing a new category name; delete button on the bottom-left when editing)
- **Backend API (Axum 0.8)** — base `/api/v1/db/dbpg006` ([`dbpg006_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/dbpg006_router.rs)):
  - `GET /languages`, `GET /categories`
  - `GET /labels` (`?category=&search=&page=&limit=`, default 25, max 200)
  - `GET /labels/entry?category=&key=`
  - `POST /labels`, `PUT /labels` (body `{category, key, row_version, values: [{language_id, value}]}`)
  - `DELETE /labels?category=&key=` (`204 No Content`)
  - Error responses: Validation `400`, NotFound `404`, Duplicate `409`, VersionConflict `412`, Database `500`
- **Application Layer**: [`features/db/dbpg006/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/db/dbpg006/) (`Dbpg006Queries`, `Dbpg006Commands`, `Dbpg006CommandExecutor`)
  - Pagination operates on distinct `(category, key)` pairs (`GROUP BY` in SQL) and then loads all language rows for the pairs on that page; `total` counts logical entries rather than raw rows.
- **Domain Layer**: `domain::entity::{db_label, db_language, db_language_label}`
- **Tests**: `application/master/tests/dbpg006_test.rs`, `handler/api/master/tests/dbpg006_router_test.rs`
