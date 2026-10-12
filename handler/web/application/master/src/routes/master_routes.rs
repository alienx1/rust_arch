//! Master SPA route tree (Angular-style nested routing)
//!
//! - `/login`                       → standalone login page (no shell)
//! - `/settings`                    → standalone set-password page (Kratos recovery / settings flow)
//! - `/` (`MasterLayout`)           → persistent sidebar + topbar, children render in `<Outlet/>`
//!   - `/`                          → SYPG000 Dashboard
//!   - `/su/supg005`                → SUPG005 IAM & Session Monitor
//!   - `/db/dbpg000`                → DBPG000 Organizations & Companies (list)
//!   - `/db/dbpg001/org[/:id]`      → DBPG001 Organization form
//!   - `/db/dbpg001/company[/:id]`  → DBPG001 Company form
//!   - `/db/dbpg002`                → DBPG002 Branches & Departments (list)
//!   - `/db/dbpg002/branch[/:id]`   → DBPG002 Branch form
//!   - `/db/dbpg002/department[/:id]` → DBPG002 Department form
//!   - `/db/dbpg003`                → DBPG003 Employees (list)
//!   - `/db/dbpg003/employee[/:id]` → DBPG003 Employee form
//!   - `/db/dbpg004`                → DBPG004 Reference Data (list)
//!   - `/db/dbpg004/lookup[/:id]`   → DBPG004 Reference value form
//!   - `/db/dbpg006`                → DBPG006 i18n Labels (list)
//!   - `/db/dbpg006/label`          → DBPG006 Label form (`?category=&key=` edits an entry)
//!   - `/db/dbpg005`                → DBPG005 Geography (lists)
//!   - `/db/dbpg005/:level[/:id]`   → DBPG005 Geography form (province|district|sub-district|postal-code)
//!   - `/su/supg001`                → SUPG001 Systems & Modules (list)
//!   - `/su/supg001/:kind[/:id]`    → SUPG001 System / Module form
//!   - `/su/supg002`                → SUPG002 Menus & Programs (list)
//!   - `/su/supg002/program[/:id]`  → SUPG002 Program form
//!   - `/su/supg003`                → SUPG003 Permission Matrix (groups)
//!   - `/su/supg003/group[/:id]`    → SUPG003 Group form; `/su/supg003/group/:id/matrix` → matrix
//!   - `/su/supg004`                → SUPG004 User Accounts
//!   - `/su/supg004/user[/:id]`     → SUPG004 Account form
//!   - `/al/alpg001`                → ALPG001 Process Monitor (background processes, reprocess)
//!   - `/nt/ntpg000`                → NTPG000 Notifications (list; the bell is in the topbar)
//!   - `/rp/rppg000`                → RPPG000 My reports (list, preview, download)
//!   - `/al/alpg000`                → ALPG000 Audit Log (list); `/al/alpg000/log/:id` → event detail
//!   - `/demo`                      → UI Components Demo (dev only)
//!   - `/*any`                      → 404 inside the shell
//!
//! Program routes are `/{system}/{program_code}` in lower case, matching `db_program.path`
//! (see `program_paths`).

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::layouts::MasterLayout;
use crate::pages::{
    DashboardPage, Dbpg000Page, Dbpg001CompanyPage, Dbpg001OrgPage, Dbpg002BranchPage,
    Dbpg002DepartmentPage, Dbpg002Page, Dbpg003EmployeePage, Dbpg003Page, Dbpg004LookupPage, Dbpg004Page, Dbpg005FormPage, Dbpg005Page, Dbpg006LabelPage, Dbpg006Page, Supg001FormPage, Supg001Page, Supg002Page, Supg002ProgramPage, Supg003GroupPage, Supg003MatrixPage, Supg003Page, Supg004Page, Supg004UserPage, DemoPage,
    Alpg000LogPage, Alpg000Page, Alpg001Page, MasterLoginPage, MasterSettingsPage, NotFoundPage, Ntpg000Page, Rppg000Page, Supg005Page,
};

#[component]
pub fn MasterRoutes() -> impl IntoView {
    view! {
        <Routes fallback=NotFoundPage>
            <Route path=path!("/login") view=MasterLoginPage />
            <Route path=path!("/settings") view=MasterSettingsPage />
            <ParentRoute path=path!("") view=MasterLayout>
                <Route path=path!("") view=DashboardPage />
                <Route path=path!("su/supg005") view=Supg005Page />
                <Route path=path!("db/dbpg000") view=Dbpg000Page />
                // DBPG001's menu entry opens the DBPG000 list; a catalogue row that still carries
                // `/db/dbpg001` as its path shows it here (same URL, so the entry stays active)
                <Route path=path!("db/dbpg001") view=Dbpg000Page />
                <Route path=path!("db/dbpg001/org") view=Dbpg001OrgPage />
                <Route path=path!("db/dbpg001/org/:id") view=Dbpg001OrgPage />
                <Route path=path!("db/dbpg001/company") view=Dbpg001CompanyPage />
                <Route path=path!("db/dbpg001/company/:id") view=Dbpg001CompanyPage />
                <Route path=path!("db/dbpg002") view=Dbpg002Page />
                <Route path=path!("db/dbpg002/branch") view=Dbpg002BranchPage />
                <Route path=path!("db/dbpg002/branch/:id") view=Dbpg002BranchPage />
                <Route path=path!("db/dbpg002/department") view=Dbpg002DepartmentPage />
                <Route path=path!("db/dbpg002/department/:id") view=Dbpg002DepartmentPage />
                <Route path=path!("db/dbpg003") view=Dbpg003Page />
                <Route path=path!("db/dbpg003/employee") view=Dbpg003EmployeePage />
                <Route path=path!("db/dbpg003/employee/:id") view=Dbpg003EmployeePage />
                <Route path=path!("db/dbpg004") view=Dbpg004Page />
                <Route path=path!("db/dbpg004/lookup") view=Dbpg004LookupPage />
                <Route path=path!("db/dbpg004/lookup/:id") view=Dbpg004LookupPage />
                <Route path=path!("db/dbpg005") view=Dbpg005Page />
                <Route path=path!("db/dbpg005/:level") view=Dbpg005FormPage />
                <Route path=path!("db/dbpg005/:level/:id") view=Dbpg005FormPage />
                <Route path=path!("db/dbpg006") view=Dbpg006Page />
                <Route path=path!("db/dbpg006/label") view=Dbpg006LabelPage />
                <Route path=path!("su/supg001") view=Supg001Page />
                <Route path=path!("su/supg001/:kind") view=Supg001FormPage />
                <Route path=path!("su/supg001/:kind/:id") view=Supg001FormPage />
                <Route path=path!("su/supg002") view=Supg002Page />
                <Route path=path!("su/supg002/program") view=Supg002ProgramPage />
                <Route path=path!("su/supg002/program/:id") view=Supg002ProgramPage />
                <Route path=path!("su/supg003") view=Supg003Page />
                <Route path=path!("su/supg003/group") view=Supg003GroupPage />
                <Route path=path!("su/supg003/group/:id") view=Supg003GroupPage />
                <Route path=path!("su/supg003/group/:id/matrix") view=Supg003MatrixPage />
                <Route path=path!("su/supg004") view=Supg004Page />
                <Route path=path!("su/supg004/user") view=Supg004UserPage />
                <Route path=path!("su/supg004/user/:id") view=Supg004UserPage />
                <Route path=path!("al/alpg000") view=Alpg000Page />
                <Route path=path!("al/alpg001") view=Alpg001Page />
                <Route path=path!("al/alpg000/log/:id") view=Alpg000LogPage />
                <Route path=path!("nt/ntpg000") view=Ntpg000Page />
                <Route path=path!("rp/rppg000") view=Rppg000Page />
                <Route path=path!("demo") view=DemoPage />
                <Route path=path!("*any") view=NotFoundPage />
            </ParentRoute>
        </Routes>
    }
}
