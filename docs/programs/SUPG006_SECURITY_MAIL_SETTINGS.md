---
id: "019ca1ed-0000-7000-8003-000000000026"
code: "SUPG006"
system_code: "SU"
module_code: "SUMT"
route_path: "/su/supg006"
icon: "LockOutlined"
title_en: "Security & Mail Settings"
parent_group: "SUPG000"
status: "Implemented"
tags:
  - program
  - security
  - settings
  - mail
  - screen
---

# 🔒 SUPG006 - Security & Mail Settings

> [!INFO] **Program Metadata**
> - **Program Code**: `SUPG006` • **Route**: `/su/supg006` • **System/Module**: `SU` / `SUMT`
> - **API**: `GET|PUT /api/v1/su/supg006/config` (`READ` to view, `WRITE` to save)
> - **Data Source**: `su_configuration` table, restricted strictly to keys allowlisted by this program (`features/su/supg006/models/config_settings.rs`)

## Goal
Allow administrators to configure brute-force sign-in protection, password complexity policies, and password email templates directly from the web UI instead of editing SQL.

## User Interface (3 Tabs)
1. **Brute-Force Sign-In Protection** — 4 parameters: max failed attempts (`security.login.max_failed_attempts`, `0` = disable lockout), attempt counting window in minutes (`…attempt_window_minutes`), lock duration in minutes (`…lock_minutes`), and minimum cooldown in minutes between password email resends (`security.password.mail_resend_minutes`). Invalid values highlight in red immediately.
2. **Password Policy** — minimum length (`security.password.min_length`, `8`–`128`; Kratos rejects anything shorter than 8) plus 5 boolean rules: require uppercase / lowercase / digit / symbol (`require_uppercase|lowercase|digit|symbol`) and forbid including the username or email local-part inside the password (`forbid_identifier`, checked when the identifier is ≥ 3 chars). Default: 8 chars + uppercase + lowercase + digit + forbid identifier (symbol not required by default).
3. **Email Templates** — switch between four template sets (`setup` initial password setup, `reset` manual reset link, `forced` admin-forced password change, `required` auto-resent when attempting to sign in while password change is required). Edit subject, HTML body, plain-text body, and sender app title, with a live side-by-side HTML preview (rendered inside a sandboxed `iframe` with scripts/links disabled) and plain-text preview populated with sample data. Supported template variables: `{{username}}`, `{{link}}`, `{{hours}}`, `{{app_name}}`.

The **Save** button requires two-step confirmation (`SaveConfirmButton`) and submits only modified keys; "Discard Changes" resets unsaved edits.

## Where the Password Policy Is Enforced
Because Ory Kratos only natively enforces minimum password length in its static config file, password complexity is enforced by the **auth gateway** (`/api/v1/auth/kratos`) before forwarding requests to Kratos: any `POST self-service/settings` submission containing a `password` field (both the change-password screen and the email recovery link flow) is validated against the active policy. If validation fails, the gateway returns **HTTP 400** with the Kratos flow JSON and a field-level error message (ID `4010003`) on the `password` node listing which rules failed (localized in Thai/English), without forwarding the weak password to Kratos; if the policy cannot be loaded from the database, the gateway fails closed with `503`.
- Existing passwords are not retroactively invalidated, and passwords set directly via Kratos' internal Admin API (e.g. `make seed-admin`) bypass the gateway.
- If `/self-service/*` traffic is not routed through the auth gateway in production, both password complexity checks and brute-force lockout will be bypassed.
- Not yet implemented: password expiration age and password history reuse prevention.

## Validation Rules (Enforced on Both Client & API)
- Only registered allowlisted keys are accepted — `mail.smtp_url` and `mail.from` (which may contain SMTP credentials) are **never exposed** on this screen.
- Numeric ranges: integers within bounds (max failed attempts `0`–`100`, attempt window `1`–`1440`, lock duration `1`–`10080`, resend cooldown `1`–`1440`, password min length `8`–`128`); boolean flags accept `1`/`0`/`true`/`false` — lock duration cannot be `0`.
- Templates: only known `{{…}}` placeholders are permitted; non-empty HTML or plain-text templates must include `{{link}}` (otherwise recipients would have no way to set their password); subject and app title must be single-line; length caps apply (subject 200, HTML 100,000, plain text 20,000 chars); leaving a template field blank falls back to the built-in default template.
- All-or-nothing save (single database transaction); any single validation error returns `400` without modifying any row.

## Integration with Other Subsystems
- **Optimistic locking**: Each configuration key carries its own `row_version` and is updated via `update_versioned`. If another admin modified a key concurrently → returns `412 VERSION_CONFLICT` (reload required); keys not yet present in the table are inserted (when `row_version = 0`), and unchanged values do not increment `row_version`.
- **Caching**: Account security checks read configuration through the `acl` cache (~60s TTL); writing to `/su/supg006` immediately invalidates the `acl` cache (`cache_policy.rs`) so changes take effect without restarting `api-master`.
- **Audit**: Both API requests and `su_configuration` row changes are recorded in `ALPG000`.

## Limitations
- Manual admin unlock button for brute-force-locked accounts is not yet implemented (locks expire automatically after `lock_minutes`).
- SMTP connection settings (`mail.smtp_url`, `mail.from`) are managed via SQL / deployment configuration.

## Related Files
- Application: `application/master/src/features/su/supg006/` • Router: `handler/api/master/src/routers/supg006_router.rs`
- Web UI: `handler/web/application/master/src/pages/su/supg006/` • Seeds: `sql/52_program_catalogue.sql`, `sql/87_su_configuration_seed.sql`, `sql/89_su_configuration_mail_templates.sql`, `sql/91_i18n_supg006.sql`
- Tests: `application/master/tests/supg006_test.rs`, `handler/api/master/tests/supg006_router_test.rs`
