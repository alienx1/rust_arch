//! Master Dashboard Main Screen Component
//!
//! Rendered inside `MainLayout`'s `<Outlet/>`; the sidebar/topbar shell is persistent.
//! Always renders the dashboard grid and tables (with empty state if data is unavailable).

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{Button, ButtonVariant, Spinner, StatusColor, StatusDot};

use crate::pages::dashboard::components::{
    ArchitectureCard, KpiCardsGrid, MasterModuleTable, ProgramCatalogue,
};
use crate::pages::dashboard::providers::{DashboardContext, provide_dashboard_context};

#[component]
pub fn DashboardScreen() -> impl IntoView {
    let ctx: DashboardContext = provide_dashboard_context();

    let on_refresh = move |_| {
        ctx.refresh.run(());
    };

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("sypg000.title", "ภาพรวมระบบจัดการข้อมูลหลัก (Master Data & IAM)")}</PageTitle>
                    <PageDescription>
                        {move || {
                            ctx.data.get()
                                .map(|d| t_fmt("sypg000.meta.region_arch", "ภูมิภาค: {region} • ระบบฐานข้อมูล: {architecture}", &[("region", &d.region.to_string()), ("architecture", &d.architecture.to_string())]))
                                .unwrap_or_else(|| t_or("sypg000.description", "แพลตฟอร์มบริหารจัดการข้อมูลองค์กร ข้อมูลบุคคลากร และการกำหนดสิทธิ์แบบรวมศูนย์ • Multi-Region"))
                        }}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || {
                        if ctx.error.get().is_some() {
                            view! { <StatusDot color=StatusColor::Destructive label="API: Disconnected" /> }.into_any()
                        } else {
                            view! { <StatusDot color=StatusColor::Success label="Online (Postgres + Ory)" /> }.into_any()
                        }
                    }}
                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9 gap-1.5"
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
                        {move || t_or("common.action.refresh", "รีเฟรชข้อมูล")}
                    </Button>
                </PageActions>
            </PageHeader>

            <PageContent>
                // Non-blocking warning banner if API is unreachable
                {move || ctx.error.get().map(|err| view! {
                    <div class="mb-4 p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-xs text-destructive flex items-center justify-between">
                        <span class="font-medium">{t_fmt("sypg000.error.api_unreachable", "ไม่สามารถเชื่อมต่อกับบริการ API แดชบอร์ดได้: {error}", &[("error", &err.to_string())])}</span>
                        <span class="text-[10px] text-muted-foreground">{t_or("sypg000.error.showing_empty", "แสดงข้อมูลว่าง")}</span>
                    </div>
                })}

                <div class="space-y-6">
                    // 4 KPI Summary Cards Grid Component
                    <KpiCardsGrid />

                    // Modules Table & Architecture Health Info Grid
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        // Left 2 cols: Master Data Modules Table & Program Catalogue
                        <div class="lg:col-span-2 space-y-4">
                            <MasterModuleTable />
                            <ProgramCatalogue />
                        </div>

                        // Right 1 col: System Info & Infrastructure Health
                        <div>
                            <ArchitectureCard />
                        </div>
                    </div>
                </div>
            </PageContent>
        </PageContainer>
    }
}
