//! System Architecture & Clean Architecture Status Card

use crate::pages::dashboard::providers::use_dashboard;
use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardDescription, CardHeader, CardTitle};

#[component]
pub fn ArchitectureCard() -> impl IntoView {
    let ctx = use_dashboard();

    view! {
        <Card class="shadow-sm border-border h-full flex flex-col justify-between">
            <div>
                <CardHeader class="pb-3">
                    <CardTitle class="text-base">{move || t_or("sypg000.card.arch.title", "โครงสร้างระบบ (Architecture)")}</CardTitle>
                    <CardDescription>{move || t_or("sypg000.card.arch.description", "สถานะเทคโนโลยีและสถาปัตยกรรม Clean Architecture")}</CardDescription>
                </CardHeader>
                <CardContent class="space-y-4 text-xs">
                    <div class="flex items-center justify-between pb-2 border-b border-border">
                        <span class="text-muted-foreground">"Core Framework"</span>
                        <span class="font-semibold text-foreground">"Rust 2024 (Leptos 0.8 CSR)"</span>
                    </div>
                    <div class="flex items-center justify-between pb-2 border-b border-border">
                        <span class="text-muted-foreground">"Design System"</span>
                        <span class="font-semibold text-foreground">"shadcn/ui + Tailwind OKLCH"</span>
                    </div>
                    <div class="flex items-center justify-between pb-2 border-b border-border">
                        <span class="text-muted-foreground">"Identity & Auth"</span>
                        <span class="font-semibold text-foreground">"Ory Kratos 26.2 (Session)"</span>
                    </div>
                    <div class="flex items-center justify-between pb-2 border-b border-border">
                        <span class="text-muted-foreground">"Database Engine"</span>
                        <span class="font-semibold text-foreground">"PostgreSQL 17 (Sea-ORM v2)"</span>
                    </div>
                    <div class="flex items-center justify-between pb-2 border-b border-border">
                        <span class="text-muted-foreground">"Cluster Architecture"</span>
                        <span class="font-semibold text-foreground">
                            {move || {
                                ctx.data.get()
                                    .map(|d| d.architecture)
                                    .unwrap_or_else(|| "Active-Active Spock pgEdge".to_string())
                            }}
                        </span>
                    </div>
                    <div class="flex items-center justify-between">
                        <span class="text-muted-foreground">"Region Gateway"</span>
                        <span class="font-semibold text-foreground font-mono text-[11px]">
                            {move || {
                                ctx.data.get()
                                    .map(|d| d.region)
                                    .unwrap_or_else(|| "ap-southeast-1".to_string())
                            }}
                        </span>
                    </div>
                </CardContent>
            </div>

            <div class="p-6 pt-4">
                <div class="p-3.5 rounded-lg bg-primary/5 border border-primary/20 text-xs space-y-1.5">
                    <div class="font-semibold text-primary flex items-center gap-1.5">
                        <span class="h-2 w-2 rounded-full bg-primary animate-pulse"></span>
                        "Clean Architecture CQRS"
                    </div>
                    <p class="text-muted-foreground text-[11px] leading-relaxed">
                        {move || t_or("sypg000.card.arch.note", "แยกเลเยอร์ domain, application, infrastructure และ handler อย่างสมบูรณ์ ไม่มีการเรียกข้ามเลเยอร์")}
                    </p>
                </div>
            </div>
        </Card>
    }
}
