//! DBPG004 Screen (`/db/dbpg004`): reference data list
//!
//! KPI cards, category filter and the paged lookup table. Renders the standard layout with an
//! empty table when the API is unreachable.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::db::dbpg004::components::{Dbpg004FilterBar, Dbpg004StatsCards, LookupTable};
use crate::pages::db::dbpg004::providers::provide_dbpg004_context;

#[component]
pub fn Dbpg004Screen() -> impl IntoView {
    let ctx = provide_dbpg004_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("dbpg004.title", "ข้อมูลอ้างอิงระบบ (Reference Data)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg004.description", "จัดการค่าอ้างอิงที่ใช้ใน Dropdown ทั่วทั้งระบบ แยกตามหมวดหมู่ พร้อมชื่อหลายภาษา • รหัสโปรแกรม: DBPG004")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || if ctx.error.get().is_some() {
                        view! { <StatusDot color=StatusColor::Destructive label="Master DB: Disconnected" /> }.into_any()
                    } else {
                        view! { <StatusDot color=StatusColor::Success label="Master DB: Connected" /> }.into_any()
                    }}
                </PageActions>
            </PageHeader>

            <PageContent>
                {move || ctx.error.get().map(|err| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm mb-4">
                        <span class="font-medium">{t_fmt("dbpg004.state.db_error", "ไม่สามารถเชื่อมต่อฐานข้อมูลได้: {error}", &[("error", err.as_str())])}</span>
                    </div>
                })}

                <div class="space-y-6">
                    <Dbpg004StatsCards />
                    <Dbpg004FilterBar />
                    <LookupTable />
                </div>
            </PageContent>
        </PageContainer>
    }
}
