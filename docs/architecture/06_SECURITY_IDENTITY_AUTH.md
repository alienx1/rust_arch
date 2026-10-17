---
title: "Platform Architecture: Identity, Authentication & Permissions"
aliases:
  - "Security Architecture"
  - "Ory Kratos Architecture"
  - "IAM and Permissions"
tags:
  - architecture
  - security
  - kratos
  - identity
  - rbac
  - abac
  - auth
author: "Platform Engineering & Architecture Team"
date_created: 2026-09-25
status: "Living Document"
version: "2.5.0"
---

# 🔐 Chapter 6: Identity, Authentication & Permissions

> [!info] Navigation
> - ⬅️ Back to: [[05_DATA_CONSISTENCY_AND_STORAGE|Data Consistency & Storage Strategies]]
> - ➡️ Next: [[07_EVENT_STREAMING_OBSERVABILITY|Event Streaming & Observability]]
> - 🧭 [[00_INDEX|Master Index]]

---

## 1. Clean Separation: AuthN vs AuthZ

The platform enforces a strict architectural boundary between **Authentication (AuthN)** and **Authorization (AuthZ)**:

```text
┌────────────────────────────────────────┐       ┌────────────────────────────────────────┐
│     AUTHENTICATION (AuthN) - ORY KRATOS │       │    AUTHORIZATION (AuthZ) - AXUM & DB   │
├────────────────────────────────────────┤       ├────────────────────────────────────────┤
│ • Headless Identity Store              │       │ • Domain Permissions Engine            │
│ • User Credentials (Argon2id)          │       │ • Multi-Tier Scopes (Org/Company/Dept) │
│ • Login, Registration, MFA Flows       │       │ • RBAC & ABAC Policy Enforcement       │
│ • Session Cookie & Token Issuance      │       │ • User-to-Employee Mapping (`su_user`) │
│ • Account Verification & Recovery      │       │ • Program Action Checks (Read/Write)   │
└────────────────────────────────────────┘       └────────────────────────────────────────┘
```

---

## 2. Ory Kratos Architecture & Flows

### 2.1 Headless Self-Service Pattern
Ory Kratos does **not** render server-side HTML pages. It provides HTTP APIs that return JSON payloads containing UI form schemas. The **Rust Leptos WASM SPA** dynamically renders form inputs, CSRF tokens, and validation messages based on Kratos JSON responses.

```mermaid
sequenceDiagram
    autonumber
    actor User as 👤 User
    participant Leptos as 💻 Leptos SPA
    participant Kratos as 🔐 Ory Kratos (Public API)

    User->>Leptos: Navigate to /login
    Leptos->>Kratos: GET /self-service/login/browser
    Kratos-->>Leptos: 200 OK (Flow JSON with CSRF token & form fields)
    Note over Leptos: Dynamically renders form<br/>using shadcn/ui components
    
    User->>Leptos: Fills Email & Password -> Submit
    Leptos->>Kratos: POST /self-service/login?flow=... (FormData)
    Note over Kratos: Verifies Argon2id Hash<br/>Generates Session Token
    Kratos-->>Leptos: 200 OK + Set-Cookie: ory_kratos_session
    Leptos->>User: Redirect to Dashboard
```

### 2.2 Argon2id Parameter Specification
Kratos uses the OWASP-recommended Argon2id parameters:
- **Algorithm**: `argon2id` (Version 19)
- **Memory Cost ($m$)**: `19456 KiB` (19 MiB)
- **Time Cost ($t$)**: `2` iterations
- **Parallelism ($p$)**: `1` lane

---

## 3. Session Verification in Axum Middleware

### 3.1 Middleware Execution Flow
1. Incoming HTTP requests pass through `KratosAuthMiddleware`.
2. The middleware extracts the `ory_kratos_session` cookie (or `Authorization: Bearer <token>` header).
3. The middleware issues a sub-millisecond local HTTP request to `GET http://kratos:4433/sessions/whoami` with the forwarded cookie.
4. If valid, Kratos returns the identity payload:
   ```json
   {
     "id": "019ca1ec-0000-7000-8010-000000000001",
     "active": true,
     "traits": {
       "email": "admin@arch.local",
       "name": { "first": "ผู้ดูแล", "last": "ระบบ" }
     }
   }
   ```
5. Axum injects an `AuthenticatedUser` struct into request extensions, making the actor available to all subsequent handlers.

### 3.2 Axum Middleware Code Pattern
```rust
// infrastructure/src/middleware/auth/kratos.rs
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use axum_extra::extract::CookieJar;

pub async fn kratos_auth_middleware(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let session_cookie = jar.get("ory_kratos_session")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Call Kratos /sessions/whoami...
    let client = reqwest::Client::new();
    let res = client.get("http://kratos:4433/sessions/whoami")
        .header("Cookie", format!("ory_kratos_session={}", session_cookie.value()))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if res.status().is_success() {
        // Attach user context to request extensions
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

---

## 4. Permission Model & Database Hierarchy

Identity in Kratos is mapped to the internal database authorization tables:

```mermaid
erDiagram
    KRATOS_IDENTITY ||--|| su_user : "Maps via UUIDv7 ID"
    su_user ||--|| db_employee : "Assigned to"
    su_user }|--|| su_permission_group : "Member of"
    su_permission_group ||--|{ su_permission : "Grants"
    su_permission }|--|| db_program : "Action on"
    su_permission }|--|| db_master_lookup : "Action Type (READ/WRITE/APPROVE)"

    su_user {
        uuid id PK "Matches Kratos Identity ID"
        uuid employee_id FK
        uuid permission_group_id FK
        varchar username
        boolean active
    }

    su_permission_group {
        uuid id PK
        varchar scope_level "organization | company | branch | department"
        uuid organization_id FK
        varchar code "e.g. ADMIN, ACCOUNTANT"
    }

    su_permission {
        uuid id PK
        uuid permission_group_id FK
        uuid program_id FK
        uuid master_lookup_id FK "READ | WRITE | APPROVE"
        boolean active
    }
```

---

## 5. Cross-Document References

- Review relational database storage: [[05_DATA_CONSISTENCY_AND_STORAGE]]
- Review event streaming for audit logs: [[07_EVENT_STREAMING_OBSERVABILITY]]
- Review network port matrix: [[09_NETWORK_PORTS_AND_SECURITY_MATRIX]]

---

## Password mail, forced change and brute-force lock

**Mail.** api-master does not talk to a mail server: it queues each mail on the Redpanda topic `mail-outbox` (key = recipient) and **api-notification** delivers it over SMTP (retrying twice, 5 s and 30 s later; a bad address or a missing `mail.smtp_url` is not retried). The topic keeps messages 1 hour (they hold one-time links) and is not indexed into Quickwit (`make topics`). Without a Redpanda client api-master falls back to sending directly. SMTP settings (`mail.smtp_url`, `mail.from` in `su_configuration`, read on every send — no restart needed). With the queue unreachable (or `mail.smtp_url` empty when sending directly) nothing is sent and the administrator gets the one-time link on screen instead; once the mail is queued the screen says "queued", and a later SMTP failure only shows in the api-notification log. Links come from the Kratos admin `POST /admin/recovery/link` (valid 24 h, single use); the user sets the password through Kratos' recovery → settings flow.

| Action (SUPG004, needs Write) | Endpoint | Effect |
|---|---|---|
| Create user | `POST /users` | account created with `su_user.must_change_password = true`, set-password mail sent |
| Send password e-mail | `POST /users/{id}/password-mail` | reset mail; the old password keeps working |
| Force password change | `POST /users/{id}/force-password-change` | flag set, sessions revoked, mail sent; sign-in refused (message 4010002) until a new password is set |

Both actions answer `{sent, sent_to (masked), mail_error, recovery_link}`; `recovery_link` is present only when the mail failed.

The flag is checked by the Kratos login hook (`/hooks/login-guard`, after the password is right; it re-mails the link at most every `security.password.mail_resend_minutes`) and cleared by the Kratos settings hook `selfservice.flows.settings.after.password` → `/hooks/password-changed`. Both hooks need `X-Webhook-Secret` (= `KRATOS_WEBHOOK_SECRET`).

**Brute force.** Kratos cannot count failures, so the web proxy sends `/self-service/*` through api-master's gateway `/api/v1/auth/kratos/*`. It counts a failure only when Kratos answers "invalid credentials" (4000006), keyed by the user name (unknown names count too, so the answer never reveals whether an account exists). After the limit the account is locked: the login page shows message 4010001 and Kratos is not asked at all, even for the right password. Counters live in Valkey under `authguard:` (not flushed by `make cache-flush`); if Valkey is down they fall back to process memory. A lookup failure refuses the attempt (fail closed). Production: the reverse proxy in front of Kratos' public API must route `/self-service/*` through the gateway, otherwise the lock is bypassed.

Settings (`su_configuration`, category `security`, edited by SQL — no screen yet; `sql/87_su_configuration_seed.sql`):

| Key | Default | Meaning |
|---|---|---|
| `mail.smtp_url` | empty | SMTP server (`smtp://host:1025`, `smtps://user:pass@host:465`, `smtp://host:587?tls=required`) |
| `mail.from` | `no-reply@arch.local` | sender |
| `security.login.max_failed_attempts` | 5 | failures before the lock (0 = lock off) |
| `security.login.attempt_window_minutes` | 15 | window the failures are counted in |
| `security.login.lock_minutes` | 15 | how long the account stays locked |
| `security.password.mail_resend_minutes` | 5 | minimum gap between automatic re-mails |

**Sign-in e-mail follows the employee's contacts.** Saving an employee's contacts (`PUT /db/dbpg003/employees/{id}/contacts`) sets Kratos `traits.email` of their account to the default e-mail contact (first one when none is default); the change is made before the contacts are committed, so an address already used by another account is refused with `409 CONFLICT` and nothing is saved. A contact list without e-mail leaves the account alone. Because the e-mail can then differ from the user name, the Kratos web hooks now also send `traits.username` and the account is found by it (apply the updated `configmap.yaml` and restart Kratos; before that, hooks fall back to the e-mail's local part).

**Mail templates.** The four password mails (`setup`, `reset`, `forced`, `required`) are templates in `su_configuration`, category `mail_template` (seed: `sql/89_su_configuration_mail_templates.sql`): `mail.template.<kind>.subject`, `.html`, `.text`, and `mail.template.app_name` (header of the mail, default `Arch`). Placeholders `{{username}}`, `{{link}}`, `{{hours}}`, `{{app_name}}`; in `.html` the values are HTML-escaped. A missing or empty row falls back to the built-in bilingual text. Templates are read through the cache (about a minute), so an edit shows up without a restart; the SMTP settings (`mail` category) are deliberately never read through the cache because they hold the password.

If a queued mail cannot be delivered after the retries (SMTP refuses, no `mail.smtp_url`, bad address), the notification service puts a `mail_failed` notification (NTPG000, link `/su/supg004`, `data = {to (masked), reason}`) into the bell of the administrator who pressed the button; the requester travels in the queued message (`MailMessage.requested_by`, set from `application_core::mail_requester`). A holder who triggers the mail by signing in has no requester, so only the log shows it.
