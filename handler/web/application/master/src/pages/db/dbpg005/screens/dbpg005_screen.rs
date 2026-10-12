//! DBPG005 Screen (`/db/dbpg005`): geography lists
//!
//! KPI counts, level tabs with province/district filters and the table of the chosen level.
//! Renders the standard layout with an empty table when the API is unreachable.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::db::dbpg005::components::{
    Dbpg005FilterBar, Dbpg005StatsCards, GeoTable, PostalTable,
};
use crate::pages::db::dbpg005::models::GeoLevel;
use crate::pages::db::dbpg005::providers::provide_dbpg005_context;

#[component]
pub fn Dbpg005Screen() -> impl IntoView {
    let ctx = provide_dbpg005_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("dbpg005.title", "จังหวัด / อำเภอ / ตำบล (Geography)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg005.description", "ข้อมูลเขตการปกครองและรหัสไปรษณีย์ สำหรับบันทึกที่อยู่ขององค์กร ลูกค้า และพนักงาน • รหัสโปรแกรม: DBPG005")}
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
                        <span class="font-medium">{t_fmt("dbpg005.state.db_error", "ไม่สามารถเชื่อมต่อฐานข้อมูลได้: {error}", &[("error", err.as_str())])}</span>
                    </div>
                })}

                <div class="space-y-6">
                    <Dbpg005StatsCards />
                    <Dbpg005FilterBar />
                    {move || match ctx.tab.get() {
                        level @ (GeoLevel::Province | GeoLevel::District | GeoLevel::SubDistrict) => {
                            view! { <GeoTable level=level /> }.into_any()
                        }
                        GeoLevel::PostalCode => view! { <PostalTable /> }.into_any(),
                    }}
                </div>
            </PageContent>
        </PageContainer>
    }
}
