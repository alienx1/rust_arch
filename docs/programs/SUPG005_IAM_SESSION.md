---
id: "019ca1ed-0000-7000-8003-000000000025"
code: "SUPG005"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg005"
icon: "SafetyCertificateOutlined"
title_en: "IAM & Session Monitor"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - iam
  - session
  - ory-kratos
  - screen
---

# 🛡️ SUPG005 - IAM & Session Monitor

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG005`
> - **Title (EN)**: IAM & Session Monitor
> - **Route Path**: `/su/supg005`
> - **API Base**: `/api/v1/su/supg005`
> - **Icon**: `SafetyCertificateOutlined`
> - **System**: `SU` | **Module**: `SUMT` | **Parent Folder**: `SUPG000`
> - **Database UUID**: `019ca1ed-0000-7000-8003-000000000025` (seed: `sql/52_program_catalogue.sql`)
> - **Backlink**: [[00_INDEX|Back to Program Catalogue (MOC)]]

---

## 🎯 Screen Overview
Displays the current user's identity and session details (Ory Kratos), data visibility scope, per-program permissions, active signed-in devices, and a system-wide user session table enabling administrators to suspend/reactivate accounts, force password changes, and revoke active sessions.

> [!NOTE] Relationship to SUPG004
> `SUPG004` (User Accounts) handles creating/editing user accounts and assigning permission groups, whereas `SUPG005` monitors live session state and executes runtime session/account actions.

## 🔌 API (Requires Kratos Session; Fails Closed with 503 if Kratos Is Unconfigured)
| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/su/supg005/users?page=&limit=` | List users along with their active devices and sessions |
| `DELETE` | `/api/v1/su/supg005/users/{id}/sessions` | Revoke all active sessions for a user |
| `POST` | `/api/v1/su/supg005/users/{id}/suspend` | Suspend user account (disables identity in Kratos + revokes sessions) |
| `POST` | `/api/v1/su/supg005/users/{id}/reactivate` | Reactivate a suspended user account |
| `DELETE` | `/api/v1/su/supg005/sessions/{id}` | Revoke a single device session |

The Kratos after-login webhook `/api/v1/hooks/login-guard` is not a user-facing program: it is invoked directly by Kratos (authenticated via `X-Webhook-Secret`), retains its dedicated path referenced in Kratos config, and shares `Supg005Commands::authorize_login`.

## 💻 Clean Architecture Implementation
- **Frontend**: [`pages/su/supg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/web/application/master/src/pages/su/supg005/) (`Supg005Page`)
- **Router**: [`supg005_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/supg005_router.rs), webhook: [`kratos_hook_router.rs`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/handler/api/master/src/routers/kratos_hook_router.rs)
- **Application**: [`features/su/supg005/`](file:///home/attaporn_j/Desktop/work/research/rust_arch_clean/application/master/src/features/su/supg005/)
- **Tests**: `application/master/tests/supg005_test.rs`, `handler/api/master/tests/supg005_router_test.rs`
