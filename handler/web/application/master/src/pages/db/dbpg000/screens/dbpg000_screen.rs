//! Organization Screen Component (DBPG000)
//!
//! Orchestrates the main layout for managing Organizations & Companies.
//! Always renders the standard screen layout with an empty table state if data is unavailable.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{Button, ButtonVariant, Spinner, StatusColor, StatusDot};

use crate::pages::db::dbpg000::components::{
    CompanyDataTable, OrgSelectorBar, OrgStatsCards,
};
use crate::pages::db::dbpg000::providers::{Dbpg000Context, provide_dbpg000_context};

#[component]
pub fn Dbpg000Screen() -> impl IntoView {
    let ctx: Dbpg000Context = provide_dbpg000_context();

    let on_refresh = move |_| {
        ctx.refresh.run(());
    };

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("dbpg000.title", "จัดการโครงสร้างองค์กรและบริษัท (Organizations & Companies)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg000.description", "ศูนย์กลางบริหารจัดการโครงสร้างกลุ่มองค์กร, นิติบุคคลในเครือ, และสายการบังคับบัญชา • รหัสโปรแกรม: DBPG000")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || {
                        if ctx.error.get().is_some() {
                            view! { <StatusDot color=StatusColor::Destructive label=t_or("dbpg000.status.disconnected", "Master DB: Disconnected") /> }.into_any()
                        } else {
                            view! { <StatusDot color=StatusColor::Success label=t_or("dbpg000.status.connected", "Master DB: Connected") /> }.into_any()
                        }
                    }}

                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9 gap-1.5 border-border"
                        on_click=Callback::new(on_refresh)
                    >
                        {move || {
                            if ctx.is_loading.get() {
                                view! { <Spinner class="h-3.5 w-3.5 text-primary" /> }.into_any()
                            } else {
                                view! {
                                    <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                                        <path d="M3 3v5h5" />
                                        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                                        <path d="M16 21h5v-5" />
                                    </svg>
                                }.into_any()
                            }
                        }}
                        {move || t_or("dbpg000.action.refresh", "รีเฟรชข้อมูล")}
                    </Button>
                </PageActions>
            </PageHeader>

            <PageContent>
                // 1. Connection warning banner (if API unreachable, but keep UI & empty table active)
                {move || {
                    ctx.error.get().map(|err| {
                        view! {
                            <div class="flex items-center justify-between p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm mb-4">
                                <div class="flex items-center gap-2">
                                    <svg class="h-4 w-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                        <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                    </svg>
                                    <span class="font-medium">{t_fmt("dbpg000.error.connect_banner", "ไม่สามารถเชื่อมต่อฐานข้อมูลได้: {error}", &[("error", &err)])}</span>
                                </div>
                            </div>
                        }
                    })
                }}

                <div class="space-y-6">
                    // 2. KPI Stats Summary Cards
                    <OrgStatsCards />

                    // 3. Organization Selector & Actions
                    <OrgSelectorBar />

                    // 4. Company Data Table (search, status chips and refresh in its header; empty state when no records)
                    <CompanyDataTable />
                </div>
            </PageContent>
        </PageContainer>
    }
}
