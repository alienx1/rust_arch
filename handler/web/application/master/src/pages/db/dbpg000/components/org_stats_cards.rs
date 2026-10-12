//! Organization and Company KPI Stats Cards

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{Card, CardContent};

use crate::pages::db::dbpg000::providers::use_dbpg000;

#[component]
pub fn OrgStatsCards() -> impl IntoView {
    let ctx = use_dbpg000();
    let summary = ctx.summary;

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            // Card 1: Total Organizations
            <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm hover:border-primary/40 transition-colors">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                            {t_or("dbpg000.stats.orgs_title", "องค์กรทั้งหมด (Organizations)")}
                        </p>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold tracking-tight text-foreground">
                                {move || summary.get().total_organizations}
                            </span>
                            <span class="text-xs text-muted-foreground">
                                {t_or("dbpg000.stats.orgs_unit", "กลุ่มบริษัท")}
                            </span>
                        </div>
                    </div>
                    <div class="p-2.5 rounded-xl bg-primary/10 text-primary">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M3 21h18M3 7v14M21 7v14M6 10h2M6 14h2M6 18h2M16 10h2M16 14h2M16 18h2M10 21V3h4v18" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 2: Total Companies
            <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm hover:border-blue-500/40 transition-colors">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                            {t_or("dbpg000.stats.companies_title", "บริษัทในเครือ (Companies)")}
                        </p>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold tracking-tight text-foreground">
                                {move || summary.get().total_companies}
                            </span>
                            <span class="text-xs text-muted-foreground">
                                {t_or("dbpg000.stats.companies_unit", "นิติบุคคล")}
                            </span>
                        </div>
                    </div>
                    <div class="p-2.5 rounded-xl bg-blue-500/10 text-blue-500">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                            <circle cx="9" cy="7" r="4" />
                            <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 3: Active Companies
            <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm hover:border-emerald-500/40 transition-colors">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                            {t_or("dbpg000.stats.active_title", "สถานะเปิดใช้งาน (Active)")}
                        </p>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold tracking-tight text-emerald-600 dark:text-emerald-400">
                                {move || summary.get().active_companies}
                            </span>
                            <span class="text-xs text-muted-foreground">
                                {t_or("dbpg000.stats.active_unit", "บริษัทที่ดำเนินงาน")}
                            </span>
                        </div>
                    </div>
                    <div class="p-2.5 rounded-xl bg-emerald-500/10 text-emerald-500">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                            <polyline points="22 4 12 14.01 9 11.01" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 4: Inactive Companies
            <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm hover:border-amber-500/40 transition-colors">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                            {t_or("dbpg000.stats.inactive_title", "ปิด/ระงับใช้งาน (Inactive)")}
                        </p>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold tracking-tight text-amber-600 dark:text-amber-400">
                                {move || summary.get().total_companies.saturating_sub(summary.get().active_companies)}
                            </span>
                            <span class="text-xs text-muted-foreground">
                                {t_or("dbpg000.stats.inactive_unit", "บริษัทพักสถานะ")}
                            </span>
                        </div>
                    </div>
                    <div class="p-2.5 rounded-xl bg-amber-500/10 text-amber-500">
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <circle cx="12" cy="12" r="10" />
                            <line x1="12" y1="8" x2="12" y2="12" />
                            <line x1="12" y1="16" x2="12.01" y2="16" />
                        </svg>
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}
