//! Multi-Tier Scope Context & Company Switcher Component

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle,
};

use crate::pages::su::supg005::models::ScopeContextDto;
use crate::pages::su::supg005::providers::use_supg005;

#[component]
pub fn SessionScopeCard(scope: ScopeContextDto) -> impl IntoView {
    let ctx = use_supg005();
    let current_company_code = scope.company_code.clone();
    let available_companies = scope.available_companies.clone();

    let on_select_company = move |ev: web_sys::Event| {
        let select_el = event_target::<web_sys::HtmlSelectElement>(&ev);
        let selected_code = select_el.value();
        ctx.switch_company.run(selected_code);
    };

    view! {
        <Card class="border-border/60 shadow-sm bg-card hover:border-primary/30 transition-all">
            <CardHeader class="pb-3 border-b border-border/40">
                <div class="flex items-center justify-between">
                    <CardTitle class="text-base font-semibold flex items-center gap-2">
                        <svg class="w-4 h-4 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M3 21h18M3 7v14M21 7v14M6 11h3M6 15h3M15 11h3M15 15h3M9 3h6v4H9z" />
                        </svg>
                        {t_or("supg005.scope.title", "ขอบเขตองค์กรและกลุ่มสิทธิ์ (Scope & Role)")}
                    </CardTitle>
                    <Badge variant=BadgeVariant::Default class="bg-blue-600 hover:bg-blue-700 text-white font-mono text-xs">
                        {format!("Tier: {}", scope.scope_level.to_uppercase())}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent class="pt-4 space-y-3.5 text-xs">
                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.scope.role_group", "กลุ่มสิทธิ์หลัก (Role Group)")}</span>
                    <div class="text-right">
                        <span class="font-bold text-foreground text-xs">{scope.permission_group_name.clone()}</span>
                        <span class="block text-[11px] font-mono text-muted-foreground">({scope.permission_group_code.clone()})</span>
                    </div>
                </div>

                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.scope.organization", "องค์กร (Organization)")}</span>
                    <span class="font-medium text-foreground">{scope.organization_name.clone()}</span>
                </div>

                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.scope.branch_dept", "สาขาและแผนกต้นสังกัด")}</span>
                    <div class="text-right">
                        <span class="text-foreground font-medium block">{scope.branch_name.clone()}</span>
                        <span class="text-[11px] text-muted-foreground">{scope.department_name.clone()}</span>
                    </div>
                </div>

                <div class="pt-2 bg-muted/20 p-3 rounded-lg border border-border/50">
                    <div class="flex items-center justify-between mb-1.5">
                        <label class="font-medium text-foreground text-[11px] flex items-center gap-1.5">
                            <svg class="w-3.5 h-3.5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                            </svg>
                            {t_or("supg005.scope.switcher", "สลับขอบเขตการทำงาน (Active Company Switcher)")}
                        </label>
                        <span class="text-[10px] text-muted-foreground">"Multi-Tenant"</span>
                    </div>
                    <select
                        class="w-full h-8 text-xs bg-background border border-input rounded-md px-2.5 py-1 text-foreground focus:outline-none focus:ring-1 focus:ring-ring font-sans"
                        on:change=on_select_company
                    >
                        {available_companies.into_iter().map(|co| {
                            let is_selected = co.code == current_company_code;
                            let val = co.code.clone();
                            view! {
                                <option value=val selected=is_selected>
                                    {format!("[{}] {}", co.code, co.name)}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </div>
            </CardContent>
        </Card>
    }
}
