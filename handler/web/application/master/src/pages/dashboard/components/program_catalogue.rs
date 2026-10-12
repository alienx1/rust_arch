//! Registered Program Catalogue Component

use crate::pages::dashboard::providers::use_dashboard;
use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Card, CardContent, CardDescription, CardHeader, CardTitle,
};

#[component]
pub fn ProgramCatalogue() -> impl IntoView {
    let ctx = use_dashboard();

    view! {
        <Card class="shadow-sm border-border">
            <CardHeader class="pb-3">
                <div class="flex items-center justify-between">
                    <div>
                        <CardTitle class="text-sm font-semibold">{move || t_or("sypg000.catalogue.title", "โปรแกรมและหน้าจอระบบ (Registered Programs)")}</CardTitle>
                        <CardDescription class="text-xs">{move || t_or("sypg000.catalogue.description", "รายการโปรแกรมที่ลงทะเบียนในตาราง db_program")}</CardDescription>
                    </div>
                    <Badge variant=BadgeVariant::Secondary>
                        {move || {
                            ctx.data.get()
                                .map(|d| t_fmt("sypg000.catalogue.count", "{count} รายการ", &[("count", &d.kpis.programs.to_string())]))
                                .unwrap_or_else(|| t_fmt("sypg000.catalogue.count", "{count} รายการ", &[("count", "21")]))
                        }}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent>
                <div class="flex flex-wrap gap-2">
                    {move || {
                        if let Some(ref data) = ctx.data.get() {
                            data.recent_programs.iter().map(|p| {
                                let code = p.code.clone();
                                let label = web_package::core::i18n::localized(&p.label_th, &p.label_en);
                                view! {
                                    <div class="px-2.5 py-1.5 rounded-md bg-muted/60 border border-border/80 flex items-center gap-1.5 text-xs">
                                        <span class="font-mono font-bold text-primary">{code}</span>
                                        <span class="text-muted-foreground font-light">"•"</span>
                                        <span class="text-foreground">{label}</span>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        } else {
                            view! {
                                <span class="text-xs text-muted-foreground">{move || t_or("sypg000.catalogue.loading", "กำลังโหลดโปรแกรม...")}</span>
                            }.into_any()
                        }
                    }}
                </div>
            </CardContent>
        </Card>
    }
}
