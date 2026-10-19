# 12 — Data Visibility Scope (View Scope) and Company / Branch Switcher

How much organizational data a user can see depends on **the branch to which the account's employee belongs** and **the permission group assignments granted to the account** (`su_user_assignment`) — allowing a single user to access **multiple branches** even though their employee record belongs to a single home branch.

| Account | What They See | Switcher Dropdown |
|---|---|---|
| Belongs to the **HQ** branch (`db_branch.code = 'HQ'`) or holds a permission group assigned at the **Organization level** | Entire organization (or the selected company/branch) | Can select any company/branch + "All Companies" / "All Branches" |
| Other accounts | **Own home branch** + everything covered by assigned permission groups (see below) | Can select only accessible companies/branches; locked if only one option is available |

Active permission groups held by the account expand visibility as follows (multiple assignments combine as a union):

| Assigned Scope Level | Additional Visibility |
|---|---|
| Organization | Entire organization (same as HQ) |
| Company | Every branch within that company (full branch access) |
| Branch | That specific branch (full branch access) |
| Department | **Only that department** — the parent branch row is visible, but other departments and employees in that branch are hidden |

Accounts without a linked employee record (none linked or employee deactivated) are not restricted by view scope — they are still governed by normal per-program permissions.

> View scope is **additive to** per-program permissions (`SUPG003`/`SUPG004`), not a replacement: the user must first have permission to open the screen, and the data within that screen is then filtered by branch scope.

## How It Works

1. **Web UI** — `CompanySwitcher` in the sidebar under the organization name renders two dropdowns: Company and Branch (branch options follow the selected company; changing company clears the branch selection). Selections are stored in `localStorage` (`company_id`, `branch_id`) and trigger a page reload. Every API request sends `X-Company-Id` / `X-Branch-Id` headers (`web_package::core::company_scope`).
2. **Middleware** — `handler/api/master/src/view_scope_layer.rs` (running after the Kratos session middleware) resolves the account's `Viewer` (`AccessControl::viewer`: organization/company/branch + whether HQ; cached in Valkey `acl` for 60 seconds) and computes `ViewScope` via `view_scope_for`:
   - **HQ**: `company_id = X-Company-Id` and/or `branch_id = X-Branch-Id` (must belong to the same organization, otherwise treated as unselected). If a branch is selected, **the branch determines the company** (a mismatched company header is overridden by the branch's company); no branch selected = all branches.
   - **Non-HQ**: All accessible branches (`ViewScope` = list of companies + full-access branches + partial branches/departments). Dropdown selections can **narrow** the scope only within what the user already has access to; out-of-scope IDs sent in headers are ignored.
   - **Viewer lookup failure** → returns HTTP 503 (**never** fails open to full visibility on error).
3. `ViewScope` is stored in a task-local (`application_core::view_scope`, similar to request language) — filtered queries call `view_scope_filters::{by_company, by_branch, in_scope}` (`application/master/src/features/db/view_scope_filters.rs`).
4. Valkey `org` and `dashboard` caches are partitioned by scope (`ReadCache` includes `view_scope::signature()` in the cache key) — differently scoped responses never share cache entries.

`GET /api/v1/access/me` also returns: `is_hq`, `companies` / `branches` (dropdown options: entire organization for HQ, accessible items only for others), and `selected_company_id` / `selected_branch_id` (clamped by the server; `null` = all). The web UI uses these fields to render the switcher dropdowns and the company subtitle beneath the organization name in the sidebar.

## Per-Screen Permissions Are Bound to the Branches Granting That Screen

A user may hold multiple permission groups across different branches where each group grants different programs (for example, `ABAC_TEST_DEPT` @ HQ has no Dashboard access, while `ABAC_TEST_DEPT_X` @ B001 does). Therefore, when loading a screen (`GET` on a specific program), the middleware computes the scope using **only the groups that grant `READ` on that program** (`AccessControl::granting_groups`):

- Data on that screen comes only from branches covered by those granting groups (a home branch without permission for that program is excluded) — this also applies to HQ users, unless they hold an organization-level assignment for that program (which still sees the entire organization).
- If the user selects a branch/company in the dropdown that falls outside the granting scope for that screen → returns **HTTP 403 `FORBIDDEN`** (rather than silently ignoring the selection). For example, switching to a branch that lacks Dashboard permission blocks the Dashboard with 403 while other screens permitted at that branch continue to work.
- Program-agnostic routes (`/access/me`, sidebar menu, notifications) and write requests (`POST`/`PUT`/`DELETE`) are not narrowed by granting group — writes continue to be enforced by `require_unit` against the target entity's scope.
- Screens accessed via another program's menu entry (e.g. organization list at `/db/dbpg000` opened from menu `DBPG001`) have no direct group grant on that path → not narrowed by group, and `require_any` authorizes access as usual.
- The sidebar menu continues to show any program the user can access in at least one branch (it does not hide menu items when switching branches); opening a screen not permitted in the currently selected branch returns 403.

## Filtered Screens

`DBPG000` (company list + KPIs) · `DBPG002` (branches, departments, company options + KPIs) · `DBPG003` (employees, company/branch/department options + KPIs) · Dashboard (company/branch/department/employee counts) — including direct lookup by ID (out of scope = 404).

## Unfiltered Areas & Limitations

- Write operations (create/update/delete) still enforce permission group scopes (`SUPG003`/`SUPG004` via `require_unit`) directly rather than `ViewScope`.
- Organization list (`DBPG000` organizations), `SUPG` screens (users/groups/permissions), and reference/geography data are not filtered by view scope.
- "HQ" is determined by branch code `HQ` (`HQ_BRANCH_CODE`); there is no separate boolean column.
- Switching company/branch reloads the page (unsaved form state is discarded).
- Assigned permission groups are read through the 60-second `acl` cache: updating user assignments in `SUPG004` flushes the cache immediately, whereas transferring an employee to a different branch takes effect within 60 seconds.
- A stale `company_id` or `branch_id` left in `localStorage` that is no longer valid is ignored by the server (falling back to the overview).
