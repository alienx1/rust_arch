---
id: "019ca1ed-0000-7000-8003-000000000024"
code: "SUPG004"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg004"
icon: "UserOutlined"
title_en: "User Accounts"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - iam
  - users
  - ory-kratos
  - screen
---

# 👥 SUPG004 - User Accounts

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG004`
> - **Title (EN)**: User Accounts
> - **Route Path**: `/su/supg004`
> - **Icon**: `UserOutlined` (`icondata_ai`)
> - **System**: `SU` (Security)
> - **Module**: `SUMT` (Security Maintenance)
> - **Parent Folder**: `SUPG000` (Security)
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000024`
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 1. Screen Overview
Screen for managing ERP user accounts (`su_user`), assigning permission groups (`su_permission_group`) with per-user organizational scopes (`su_user_assignment`), managing account active/suspension states, and coordinating with **Ory Kratos 1.3** to provision identities and issue password setup / recovery links.

---

## 📝 2. Scope & Requirements

### 2.1 Objectives
- Create new user accounts linked to employees, assign role & organizational scope combinations, issue password recovery links, and activate/suspend accounts.

### 2.2 Fields & Schema
- [x] **Account Information (`su_user`)**:
  - Username: sign-in identifier
  - Email: used for notifications and Ory Kratos recovery flows
  - Linked Employee: references `db_employee`
- [x] **Role & Scope Assignments (`su_user_assignment`)**:
  - Permission Groups: a user can hold multiple assignments
  - Organizational Scope per Assignment: Organization, Company, Branch, or Department
- [x] **Security Status**:
  - Active status (`active` boolean)
  - Password change required flag & brute-force lockout state
  - Password recovery / setup link generation via Ory Kratos

### 2.3 Business Rules & Actions
- [x] **Ory Kratos Identity Sync**: Creating a new user account calls the Ory Kratos Admin API to provision the identity with schema traits and compensates (deletes the Kratos identity) if the database transaction fails.
- [x] **Password Reset / Recovery Link**: Never stores plaintext passwords; issues a one-time Kratos recovery link (and dispatches an email via `mail-outbox` when SMTP is configured).
- [x] **Session Revocation on Suspension**: Deactivating an account immediately disables the Kratos identity and revokes all active sessions.

---

## 🔒 3. Permissions & Security
- **Permissions defined in `su_permission`**:
  - `READ`: Search and view user accounts
  - `WRITE`: Create user accounts, update details, assign non-admin permission groups, and issue recovery links
  - `APPROVE`: Required to delete user accounts (`DELETE`) or to assign/remove a user in the `ADMIN` permission group

---

## 🗄️ 4. Database Entities
- `su_user` (`id`, `username`, `email`, `employee_id`, `permission_group_id`, `active`, `row_version`, ...)
- `su_user_assignment` (`id`, `user_id`, `permission_group_id`, `organization_id`, `company_id`, `branch_id`, `department_id`, ...)
- `db_employee` (linked employee profile)
- Ory Kratos `identities` table (managed in Kratos schema)

---

## 💻 5. Clean Architecture Implementation
- **Frontend (Leptos)**: `pages/su/supg004/` (server-paged user table, `UserForm` with employee/email/username/assignment editors, `RecoveryLinkPanel`) • routes `/su/supg004`, `/su/supg004/user[/{id}]`
- **Backend API** (`/api/v1/su/supg004`): `GET /users`, `GET|PUT|DELETE /users/{id}`, `POST /users` (provisions Kratos identity + `su_user` row + password setup link), `POST /users/{id}/recovery-link`, `GET /options/employees`, `GET /options/groups`, `GET /scope-options`
- **Application**: `features/su/supg004/` — `IdentityProvisioningPort` (`application/core`) implemented by `KratosConfig` (Kratos Admin API: `/admin/identities`, `/admin/recovery/link`)
- **Scoped Role Assignments (`su_user_assignment`)**: A single user can hold multiple assignments, where each assignment pairs 1 permission group with 1 organizational unit (organization / company / branch / department) — e.g. Group A at Branch 1 + Group B at Branch 2, keeping permission groups in `SUPG003` purely about capabilities. Every user must have at least 1 assignment, and the first assignment sets the primary `su_user.permission_group_id`. Updates are applied as a diff within a single transaction. Assignments drive both `AccessControl::authorize_in` (write/delete scope checks in `DBPG002`/`DBPG003`) and `ViewScope` (read filtering — see [[12_VIEW_SCOPE_COMPANY_SWITCHER]]), while the sidebar menu and `/access/me` union grants across all active assignments.
- **Deleting an Account**: `DELETE /users/{id}` (`204 No Content`) permanently deletes the `su_user` row and then revokes sessions and deletes the Kratos identity (best-effort; logs a warning on Kratos cleanup failure). Users cannot delete their own account (`403`); if referenced by other records, returns `409 IN_USE` (suspend the account instead).
- **Key Guardrails**: Only employees without an existing account can be selected when creating a user; if DB insertion fails, the newly created Kratos identity is rolled back; assigning or modifying membership in the `ADMIN` group requires `APPROVE` permission; users cannot suspend themselves or strip their own access to `SUPG004`; account suspension/reactivation delegates to `Supg005Commands` (revoking active sessions).
- **Tests**: `supg004_test.rs`, `handler/api/master/tests/supg004_router_test.rs`
