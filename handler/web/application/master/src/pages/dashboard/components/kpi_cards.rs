use crate::pages::dashboard::providers::use_dashboard;
use leptos::prelude::*;
use web_package::core::auth::use_auth;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle, Spinner,
};

#[component]
pub fn KpiCardsGrid() -> impl IntoView {
    let auth = use_auth();
    let ctx = use_dashboard();

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            // Card 1: Companies & Branches
            <Card class="shadow-sm border-border hover:shadow-md transition-shadow">
                <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0">
                    <CardTitle class="text-xs font-medium text-muted-foreground">{move || t_or("sypg000.kpi.org_branch", "องค์กรและสาขา")}</CardTitle>
                    <Badge variant=BadgeVariant::Success>"db_company"</Badge>
                </CardHeader>
                <CardContent>
                    {move || {
                        if ctx.is_loading.get() && ctx.data.get().is_none() {
                            view! { <div class="py-2"><Spinner class="h-5 w-5 text-primary" /></div> }.into_any()
                        } else {
                            let companies = ctx.data.get().map(|d| d.kpis.companies).unwrap_or(0);
                            let branches = ctx.data.get().map(|d| d.kpis.branches).unwrap_or(0);
                            let orgs = ctx.data.get().map(|d| d.kpis.organizations).unwrap_or(0);
                            view! {
                                <div>
                                    <div class="text-2xl font-bold text-foreground">{t_fmt("sypg000.kpi.companies_count", "{count} บริษัท", &[("count", &companies.to_string())])}</div>
                                    <p class="text-xs text-muted-foreground mt-1">
                                        {t_fmt("sypg000.kpi.branches_orgs", "{branches} สาขา • {orgs} องค์กรหลัก", &[("branches", &branches.to_string()), ("orgs", &orgs.to_string())])}
                                    </p>
                                </div>
                            }.into_any()
                        }
                    }}
                </CardContent>
            </Card>

            // Card 2: Employees & Departments
            <Card class="shadow-sm border-border hover:shadow-md transition-shadow">
                <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0">
                    <CardTitle class="text-xs font-medium text-muted-foreground">{move || t_or("sypg000.kpi.emp_dept", "พนักงานและฝ่ายงาน")}</CardTitle>
                    <Badge variant=BadgeVariant::Secondary>"db_employee"</Badge>
                </CardHeader>
                <CardContent>
                    {move || {
                        if ctx.is_loading.get() && ctx.data.get().is_none() {
                            view! { <div class="py-2"><Spinner class="h-5 w-5 text-primary" /></div> }.into_any()
                        } else {
                            let employees = ctx.data.get().map(|d| d.kpis.employees).unwrap_or(0);
                            let depts = ctx.data.get().map(|d| d.kpis.departments).unwrap_or(0);
                            view! {
                                <div>
                                    <div class="text-2xl font-bold text-foreground">{t_fmt("sypg000.kpi.employees_count", "{count} คน", &[("count", &employees.to_string())])}</div>
                                    <p class="text-xs text-muted-foreground mt-1">
                                        {t_fmt("sypg000.kpi.depts_count", "{count} ฝ่าย/แผนกงาน (HR Linked)", &[("count", &depts.to_string())])}
                                    </p>
                                </div>
                            }.into_any()
                        }
                    }}
                </CardContent>
            </Card>

            // Card 3: Geography & Localization
            <Card class="shadow-sm border-border hover:shadow-md transition-shadow">
                <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0">
                    <CardTitle class="text-xs font-medium text-muted-foreground">{move || t_or("sypg000.kpi.geo_i18n", "ข้อมูลภูมิศาสตร์และภาษา")}</CardTitle>
                    <Badge variant=BadgeVariant::Outline>"db_province"</Badge>
                </CardHeader>
                <CardContent>
                    {move || {
                        if ctx.is_loading.get() && ctx.data.get().is_none() {
                            view! { <div class="py-2"><Spinner class="h-5 w-5 text-primary" /></div> }.into_any()
                        } else {
                            let provinces = ctx.data.get().map(|d| d.kpis.provinces).unwrap_or(0);
                            let districts = ctx.data.get().map(|d| d.kpis.districts).unwrap_or(0);
                            let labels = ctx.data.get().map(|d| d.kpis.labels).unwrap_or(0);
                            let languages = ctx.data.get().map(|d| d.kpis.languages).unwrap_or(0);
                            view! {
                                <div>
                                    <div class="text-2xl font-bold text-foreground">{t_fmt("sypg000.kpi.provinces_count", "{count} จังหวัด", &[("count", &provinces.to_string())])}</div>
                                    <p class="text-xs text-muted-foreground mt-1">
                                        {t_fmt("sypg000.kpi.districts_labels", "{districts} อำเภอ • {labels} คำศัพท์ ({languages} ภาษา)", &[("districts", &districts.to_string()), ("labels", &labels.to_string()), ("languages", &languages.to_string())])}
                                    </p>
                                </div>
                            }.into_any()
                        }
                    }}
                </CardContent>
            </Card>

            // Card 4: IAM Session (Ory Kratos)
            <Card class="shadow-sm border-border hover:shadow-md transition-shadow">
                <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0">
                    <CardTitle class="text-xs font-medium text-muted-foreground">{move || t_or("sypg000.kpi.iam_status", "สถานะเซสชัน (IAM)")}</CardTitle>
                    {move || {
                        if auth.is_authenticated.get() {
                            view! { <Badge variant=BadgeVariant::Success>"Online"</Badge> }.into_any()
                        } else {
                            view! { <Badge variant=BadgeVariant::Warning>"Anonymous"</Badge> }.into_any()
                        }
                    }}
                </CardHeader>
                <CardContent>
                    {move || {
                        if auth.is_loading.get() {
                            view! {
                                <div class="flex items-center gap-2 py-1 text-xs text-muted-foreground">
                                    <Spinner class="h-4 w-4 text-primary" />
                                    <span>{move || t_or("common.state.loading", "กำลังโหลดข้อมูล...")}</span>
                                </div>
                            }.into_any()
                        } else if let Some(session) = auth.session.get() {
                            let email = session.email().unwrap_or_else(|| "admin@arch.local".to_string());
                            let short_id = if session.id.len() > 12 { &session.id[..12] } else { &session.id };
                            view! {
                                <div>
                                    <div class="text-sm font-semibold text-foreground truncate">{email}</div>
                                    <p class="text-[11px] text-muted-foreground mt-0.5 truncate font-mono">
                                        {format!("ID: {short_id}...")}
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    <div class="text-sm font-medium text-muted-foreground">{move || t_or("sypg000.kpi.unauthenticated", "ยังไม่ระบุตัวตน")}</div>
                                    <a href="/login" class="text-xs text-primary hover:underline">{move || t_or("common.action.login", "เข้าสู่ระบบ")}</a>
                                </div>
                            }.into_any()
                        }
                    }}
                </CardContent>
            </Card>
        </div>
    }
}
