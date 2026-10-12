//! SUPG003 Screen (`/su/supg003`): permission groups
//!
//! The group table; each group opens its matrix. Renders the standard layout with an empty table
//! when the API is unreachable (or the account lacks READ on SUPG003).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::su::supg003::components::GroupTable;
use crate::pages::su::supg003::providers::provide_supg003_context;

#[component]
pub fn Supg003Screen() -> impl IntoView {
    let ctx = provide_supg003_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("supg003.title", "แฟ้มสิทธิ์การเข้าใช้งาน (Permission Matrix)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg003.description", "จัดกลุ่มสิทธิ์ตามขอบเขตข้อมูล และกำหนดสิทธิ์ READ / WRITE / APPROVE รายโปรแกรม • รหัสโปรแกรม: SUPG003")}
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
                        <span class="font-medium">{err}</span>
                    </div>
                })}
                <GroupTable />
            </PageContent>
        </PageContainer>
    }
}
