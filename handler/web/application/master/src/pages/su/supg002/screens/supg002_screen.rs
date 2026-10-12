//! SUPG002 Screen (`/su/supg002`): menu & program management
//!
//! System / module filters and the paged program table. Renders the standard layout with an
//! empty table when the API is unreachable (or the account lacks READ on SUPG002).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::su::supg002::components::{ProgramTable, Supg002FilterBar};
use crate::pages::su::supg002::providers::provide_supg002_context;

#[component]
pub fn Supg002Screen() -> impl IntoView {
    let ctx = provide_supg002_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("supg002.title", "จัดการเมนู (Menus & Programs)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg002.description", "ลงทะเบียนโปรแกรม จัดโฟลเดอร์ ลำดับ ไอคอน และเปิด/ปิดการแสดงเมนูใน sidebar • รหัสโปรแกรม: SUPG002")}
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

                <div class="space-y-6">
                    <Supg002FilterBar />
                    <ProgramTable />
                </div>
            </PageContent>
        </PageContainer>
    }
}
