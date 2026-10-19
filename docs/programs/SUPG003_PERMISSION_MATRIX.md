---
id: "019ca1ed-0000-7000-8003-000000000023"
code: "SUPG003"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg003"
icon: "SafetyOutlined"
title_en: "Permission Matrix"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - permissions
  - rbac
  - abac
  - screen
---

# 🛡️ SUPG003 - Permission Matrix

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG003`
> - **Title (EN)**: Permission Matrix
> - **Route Path**: `/su/supg003`
> - **Icon**: `SafetyOutlined` (`icondata_ai`)
> - **System**: `SU` (Security)
> - **Module**: `SUMT` (Security Maintenance)
> - **Parent Folder**: `SUPG000` (Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000023`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Core authorization management screen for defining permission groups (`su_permission_group`) and configuring their program × action grant matrix (`READ`, `WRITE`, `APPROVE`).

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Create and manage permission groups (e.g. `ADMIN`, `HR_MANAGER`, `ACCOUNTANT`) and toggle `READ` / `WRITE` / `APPROVE` grants per program in an interactive matrix grid.

### 2.2 Fields & Schema
- [x] **Permission Group (`su_permission_group`)**:
  - Group Code: e.g. `ADMIN`, `SUPV`, `OPERATOR`
  - Active Status: `active`
- [x] **Permission Matrix Grid (`su_permission`)**:
  - Rows: System -> Module -> Program hierarchy
  - Columns: Permission actions (`READ`, `WRITE`, `APPROVE`)
  - Checkboxes: Grant or revoke each action per program (granting `WRITE` or `APPROVE` automatically requires `READ`)

### 2.3 Business Rules & Actions
- [x] **Admin Group Protection**: The `ADMIN` group automatically holds full permissions across all active programs and cannot have its matrix edited, be deactivated, or be deleted.
- [x] **Self-Lockout Prevention**: Administrators cannot revoke permissions in a way that would remove their own access to `SUPG003`.
- [x] **Bulk Grant / Revoke**: Select-all toggles allow granting or revoking permissions across an entire module or column rapidly.
- [x] **Audit Trail on Changes**: Every permission group or matrix update is recorded in Quickwit (`ALPG000`).

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: View permission groups and their permission matrix
  - `WRITE`: Create/update permission groups and save matrix changes
  - `APPROVE`: Delete permission groups (`DELETE` requires `APPROVE`)

---

## 🗄️ 4. Database Entities
- `su_permission_group` (`id`, `code`, `active`, `row_version`, ...)
- `su_permission` (`id`, `permission_group_id`, `system_id`, `module_id`, `program_id`, `master_lookup_id`, `active`, ...)
- `su_user_assignment` (binds a user to a permission group at a specific organizational scope in `SUPG004`)
- `db_master_lookup` (category `permission_type` for actions: `READ`, `WRITE`, `APPROVE`)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg003/` (group list, `GroupForm`, `MatrixTable` with checkboxes + select-all toggles) • routes `/su/supg003`, `/su/supg003/group[/{id}]`, `/su/supg003/group/{id}/matrix`
- **Backend API** (`/api/v1/su/supg003`): `GET|POST /groups`, `GET|PUT|DELETE /groups/{id}`, `GET|PUT /groups/{id}/matrix`
- **Application**: `features/su/supg003/` (`plan_changes` diffs current vs. desired grants and deactivates removed rows rather than hard-deleting; `WRITE`/`APPROVE` require `READ`)
- **Deleting a Group**: `DELETE /groups/{id}` (`204 No Content`) permanently deletes the group along with its `su_permission` rows in a single transaction; `ADMIN` cannot be deleted (`400`); groups still assigned to users return `409 IN_USE` (reassign users first or deactivate instead)
- **Optimistic Locking**: `PUT /groups/{id}/matrix` requires the group's loaded `row_version` (`SaveMatrixCommand.row_version`). The matrix is treated as part of the group aggregate: if any grant changes, `su_permission_group.row_version` is incremented by 1 via `update_versioned` (if no grants changed, the version stays unchanged); a stale version returns `412 VERSION_CONFLICT` without modifying any rows.
- **Key Guardrails**: `ADMIN` cannot have its matrix modified or be deactivated; users cannot strip their own access to `SUPG003`; groups currently assigned to users cannot be deactivated.
- **Separation of Role ("What") vs. Scope ("Where")**: `su_permission_group` stores only `code` and `active` — defining *what* actions are permitted (`program × READ/WRITE/APPROVE`). *Where* those permissions apply (organization / company / branch / department) is assigned per user in `SUPG004` via `su_user_assignment` (allowing a single user to hold Group A at Branch 1 and Group B at Branch 2).
- **Scoped Authorization**: `AccessControl::authorize_in(user, program, action, ScopeTarget)` verifies that the user holds an active assignment whose group grants `action` on `program` and whose organizational unit covers the target record (higher-level scopes cover all descendant units). Used for write/delete checks in `DBPG002` and `DBPG003`, while read visibility is governed by `ViewScope` (see [[12_VIEW_SCOPE_COMPANY_SWITCHER]]).
- **Tests**: `supg003_test.rs`, `supg003_router_test.rs`
