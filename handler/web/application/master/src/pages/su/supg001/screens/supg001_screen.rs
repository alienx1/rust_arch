//! SUPG001 Screen (`/su/supg001`): systems & modules
//!
//! Tabs for systems and modules, a system filter for modules and the chosen table. Renders the
//! standard layout with an empty table when the API is unreachable (or the account lacks READ).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::su::supg001::components::{ModulesTable, Supg001FilterBar, SystemsTable};
use crate::pages::su::supg001::models::UnitKind;
use crate::pages::su::supg001::providers::provide_supg001_context;

#[component]
pub fn Supg001Screen() -> impl IntoView {
    let ctx = provide_supg001_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("supg001.title", "จัดการระบบ / โมดูล (Systems & Modules)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg001.description", "กำหนดระบบและโมดูลที่โปรแกรมสังกัด ปิดระบบหรือโมดูลเพื่อซ่อนเมนูทั้งชุด • รหัสโปรแกรม: SUPG001")}
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
                    <Supg001FilterBar />
                    {move || match ctx.tab.get() {
                        UnitKind::System => view! { <SystemsTable /> }.into_any(),
                        UnitKind::Module => view! { <ModulesTable /> }.into_any(),
                    }}
                </div>
            </PageContent>
        </PageContainer>
    }
}
