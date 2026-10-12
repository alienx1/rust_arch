//! DBPG004 KPI Stats Cards (reference data)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent};

use crate::pages::db::dbpg004::providers::use_dbpg004;

#[component]
fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] unit: String,
    #[prop(into)] value: Signal<u64>,
    #[prop(optional)] value_class: &'static str,
) -> impl IntoView {
    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4">
                <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{label}</p>
                <div class="flex items-baseline gap-2 mt-1">
                    <span class=format!("text-2xl font-bold tracking-tight text-foreground {value_class}")>
                        {move || value.get()}
                    </span>
                    <span class="text-xs text-muted-foreground">{unit}</span>
                </div>
            </CardContent>
        </Card>
    }
}

#[component]
pub fn Dbpg004StatsCards() -> impl IntoView {
    let summary = use_dbpg004().summary;

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <StatCard
                label=t_or("dbpg004.stats.categories", "หมวดหมู่ (Categories)")
                unit=t_or("dbpg004.stats.unit_category", "หมวด")
                value=Signal::derive(move || summary.get().total_categories)
            />
            <StatCard
                label=t_or("dbpg004.stats.lookups", "ค่าอ้างอิงทั้งหมด (Lookups)")
                unit=t_or("dbpg004.stats.unit_item", "รายการ")
                value=Signal::derive(move || summary.get().total_lookups)
            />
            <StatCard
                label=t_or("dbpg004.stats.active", "เปิดใช้งาน (Active)")
                unit=t_or("dbpg004.stats.unit_item", "รายการ")
                value=Signal::derive(move || summary.get().active_lookups)
                value_class="text-emerald-600 dark:text-emerald-400"
            />
        </div>
    }
}
