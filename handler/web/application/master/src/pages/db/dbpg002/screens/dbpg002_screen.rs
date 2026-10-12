//! DBPG002 Screen (`/db/dbpg002`): branches & departments list
//!
//! KPI cards, company/search/status toolbar and the Branches / Departments tables. Renders the
//! standard layout with empty tables when the API is unreachable.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{Button, ButtonVariant, Spinner, StatusColor, StatusDot};

use crate::pages::db::dbpg002::components::{
    BranchTable, Dbpg002FilterBar, Dbpg002StatsCards, DepartmentTable,
};
use crate::pages::db::dbpg002::providers::{Dbpg002Tab, provide_dbpg002_context};

#[component]
pub fn Dbpg002Screen() -> impl IntoView {
    let ctx = provide_dbpg002_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("dbpg002.title", "กำหนดสาขา / แผนก (Branches & Departments)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg002.description", "จัดการโครงสร้างสาขาและแผนกภายใต้บริษัท สำหรับกำหนดขอบเขตข้อมูลและสิทธิ์ • รหัสโปรแกรม: DBPG002")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || if ctx.error.get().is_some() {
                        view! { <StatusDot color=StatusColor::Destructive label="Master DB: Disconnected" /> }.into_any()
                    } else {
                        view! { <StatusDot color=StatusColor::Success label="Master DB: Connected" /> }.into_any()
                    }}
                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9 gap-1.5 border-border"
                        on_click=Callback::new(move |_| ctx.refresh.run(()))
                    >
                        {move || ctx.is_loading.get().then(|| view! { <Spinner class="h-3.5 w-3.5 text-primary" /> })}
                        {move || t_or("common.action.refresh", "รีเฟรชข้อมูล")}
                    </Button>
                </PageActions>
            </PageHeader>

            <PageContent>
                {move || ctx.error.get().map(|err| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm mb-4">
                        <span class="font-medium">{t_fmt("dbpg002.error.db_connect", "ไม่สามารถเชื่อมต่อฐานข้อมูลได้: {error}", &[("error", &err)])}</span>
                    </div>
                })}

                <div class="space-y-6">
                    <Dbpg002StatsCards />
                    <Dbpg002FilterBar />
                    {move || match ctx.tab.get() {
                        Dbpg002Tab::Branches => view! { <BranchTable /> }.into_any(),
                        Dbpg002Tab::Departments => view! { <DepartmentTable /> }.into_any(),
                    }}
                </div>
            </PageContent>
        </PageContainer>
    }
}
