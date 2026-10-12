//! IAM Session Main Screen Component
//!
//! Orchestrates and renders the `/su/supg005` page (SUPG005) layout inside `MasterLayout`'s `<Outlet/>`.

use leptos::prelude::*;
use web_package::core::auth::use_auth;
use web_package::core::i18n::t_or;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{
    Button, ButtonVariant, Spinner, StatusColor, StatusDot, Tabs, TabsContent, TabsList,
    TabsTrigger,
};

use crate::pages::su::supg005::components::{
    SessionDevicesCard, SessionIdentityCard, SessionPermissionsTable, SessionRawInspector,
    SessionScopeCard, SessionSecurityCard, SessionStatsKpis, SessionUsersTable,
};
use crate::pages::su::supg005::providers::{Supg005Context, provide_supg005_context};

#[component]
pub fn Supg005Screen() -> impl IntoView {
    let ctx: Supg005Context = provide_supg005_context();
    let auth = use_auth();

    let on_refresh = move |_| {
        ctx.refresh.run(());
    };

    let on_logout = move |_| {
        auth.logout();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
        }
    };

    view! {
        <PageContainer>
            <PageHeader class="md:flex-col md:items-start xl:flex-row xl:items-center xl:justify-between">
                <div>
                    <PageTitle>{move || t_or("supg005.title", "จัดการเซสชันและการเข้าถึงระบบ (IAM Sessions & Devices)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg005.description", "ศูนย์บริหารจัดการเซสชันผู้ใช้งาน, ตรวจสอบอุปกรณ์ที่กำลังเชื่อมต่อ (Active Devices), และเพิกถอนสิทธิ์การใช้งาน (Revoke User/Device)")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || {
                        if ctx.error.get().is_some() {
                            view! { <StatusDot color=StatusColor::Destructive label=t_or("supg005.status.disconnected", "IAM: Disconnected") /> }.into_any()
                        } else if let Some(summary) = ctx.data.get() {
                            if summary.is_live_session {
                                view! { <StatusDot color=StatusColor::Success label=t_or("supg005.status.live", "Ory Kratos: Live") /> }.into_any()
                            } else {
                                view! { <StatusDot color=StatusColor::Warning label="Dev Demo: Active" /> }.into_any()
                            }
                        } else {
                            view! { <StatusDot color=StatusColor::Muted label="Checking..." /> }.into_any()
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
                        <span>{move || t_or("common.action.refresh", "รีเฟรชข้อมูล")}</span>
                    </Button>

                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9 gap-1.5 text-destructive hover:bg-destructive/10 hover:border-destructive/30"
                        on_click=Callback::new(on_logout)
                    >
                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
                            <polyline points="16 17 21 12 16 7" />
                            <line x1="21" y1="12" x2="9" y2="12" />
                        </svg>
                        <span>{move || t_or("common.action.logout", "ออกจากระบบ")}</span>
                    </Button>
                </PageActions>
            </PageHeader>

            <PageContent>
                {move || {
                    if ctx.is_loading.get() && ctx.data.get().is_none() {
                        view! {
                            <div class="flex flex-col items-center justify-center py-20 gap-3 text-muted-foreground">
                                <Spinner class="h-8 w-8 text-primary" />
                                <p class="text-sm">{t_or("supg005.screen.loading", "กำลังโหลดข้อมูลเซสชันและตรวจสอบสิทธิ์กับ Ory Kratos...")}</p>
                            </div>
                        }.into_any()
                    } else if let Some(summary) = ctx.data.get() {
                        let identity = summary.identity.clone();
                        let security = summary.security.clone();
                        let scope = summary.scope.clone();
                        let permissions = summary.permissions.clone();
                        let devices = summary.devices.clone();
                        let raw_json = summary.raw_json.clone();

                        view! {
                            <div class="space-y-6">
                                // 1. Action Notification Banner
                                {move || {
                                    if let Some(msg) = ctx.action_notification.get() {
                                        view! {
                                            <div class="p-3 bg-emerald-500/10 border border-emerald-500/25 rounded-lg text-emerald-700 dark:text-emerald-400 text-xs flex items-center justify-between shadow-sm animate-in fade-in duration-200">
                                                <div class="flex items-center gap-2">
                                                    <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                        <path d="M5 13l4 4L19 7" />
                                                    </svg>
                                                    <span class="font-medium">{msg}</span>
                                                </div>
                                                <button
                                                    type="button"
                                                    class="text-xs hover:underline cursor-pointer opacity-70 hover:opacity-100"
                                                    on:click=move |_| ctx.set_action_notification.set(None)
                                                >
                                                    {t_or("supg005.screen.close", "ปิด")}
                                                </button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }
                                }}

                                // 2. Overview Statistics KPIs
                                <SessionStatsKpis summary=summary.clone() />

                                // 3. Primary Navigation Tabs
                                <div class="pt-2">
                                    <Tabs active_tab=ctx.active_tab>
                                        <div class="flex items-center justify-between overflow-x-auto border-b border-border/40 pb-3">
                                            <TabsList class="h-9">
                                                <TabsTrigger
                                                    value="all_users".to_string()
                                                    on_select=Callback::new(move |v| ctx.set_active_tab.set(v))
                                                >
                                                    <span class="flex items-center gap-1.5 text-xs">
                                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                                                            <circle cx="9" cy="7" r="4" />
                                                            <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                                                            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                                                        </svg>
                                                        {t_or("supg005.tab.all_users", "ผู้ใช้และเซสชันทั้งหมด (All Users & Sessions)")}
                                                    </span>
                                                </TabsTrigger>

                                                <TabsTrigger
                                                    value="my_session".to_string()
                                                    on_select=Callback::new(move |v| ctx.set_active_tab.set(v))
                                                >
                                                    <span class="flex items-center gap-1.5 text-xs">
                                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
                                                            <circle cx="12" cy="7" r="4" />
                                                        </svg>
                                                        {t_or("supg005.tab.my_session", "เซสชันของฉัน (My Active Session)")}
                                                    </span>
                                                </TabsTrigger>

                                                <TabsTrigger
                                                    value="raw".to_string()
                                                    on_select=Callback::new(move |v| ctx.set_active_tab.set(v))
                                                >
                                                    <span class="flex items-center gap-1.5 text-xs">
                                                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                            <path d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                                                        </svg>
                                                        {t_or("supg005.tab.diagnostics", "ข้อมูลเชิงเทคนิค (Diagnostics JSON)")}
                                                    </span>
                                                </TabsTrigger>
                                            </TabsList>
                                        </div>

                                        // Tab 1: All Users & Sessions Directory (with Revoke User & Revoke Device)
                                        <TabsContent value="all_users".to_string()>
                                            <div class="pt-2">
                                                <SessionUsersTable />
                                            </div>
                                        </TabsContent>

                                        // Tab 2: My Active Session Details (Identity + Security + Scope + Permissions)
                                        <TabsContent value="my_session".to_string()>
                                            {
                                                let ident = identity.clone();
                                                let sec = security.clone();
                                                let sc = scope.clone();
                                                let perm = permissions.clone();
                                                let dev = devices.clone();
                                                move || view! {
                                                    <div class="pt-2 space-y-6">
                                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                                                            <SessionIdentityCard identity=ident.clone() />
                                                            <SessionSecurityCard security=sec.clone() />
                                                            <SessionScopeCard scope=sc.clone() />
                                                        </div>
                                                        <SessionPermissionsTable permissions=perm.clone() />
                                                        <SessionDevicesCard devices=dev.clone() />
                                                    </div>
                                                }
                                            }
                                        </TabsContent>

                                        // Tab 3: Raw JSON Inspector
                                        <TabsContent value="raw".to_string()>
                                            {
                                                let r = raw_json.clone();
                                                move || view! {
                                                    <div class="pt-2">
                                                        <SessionRawInspector raw_json=r.clone() />
                                                    </div>
                                                }
                                            }
                                        </TabsContent>
                                    </Tabs>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="p-12 flex flex-col items-center justify-center gap-3 text-center bg-card rounded-xl border border-border/60">
                                <div class="p-3 rounded-full bg-muted text-muted-foreground">
                                    <svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                                        <path d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                    </svg>
                                </div>
                                <h3 class="text-base font-semibold text-foreground">
                                    {t_or("supg005.screen.no_session_title", "ไม่พบข้อมูลเซสชันการเข้าสู่ระบบ")}
                                </h3>
                                <p class="text-xs text-muted-foreground max-w-sm">
                                    {ctx.error.get().unwrap_or_else(|| t_or("supg005.screen.no_session_text", "ยังไม่มีเซสชันที่เชื่อมต่อกับ Ory Kratos ในขณะนี้"))}
                                </p>
                                <a
                                    href="/login"
                                    class="inline-flex items-center justify-center rounded-md text-xs font-medium bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 mt-2"
                                >
                                    {t_or("supg005.screen.sign_in_again", "เข้าสู่ระบบใหม่")}
                                </a>
                            </div>
                        }.into_any()
                    }
                }}
            </PageContent>
        </PageContainer>
    }
}
