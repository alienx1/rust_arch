//! DBPG005 KPI Stats Cards (counts per geography level)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent};

use crate::pages::db::dbpg005::providers::use_dbpg005;

#[component]
fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] unit: String,
    #[prop(into)] value: Signal<u64>,
) -> impl IntoView {
    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4">
                <p class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{label}</p>
                <div class="flex items-baseline gap-2 mt-1">
                    <span class="text-2xl font-bold tracking-tight text-foreground">
                        {move || value.get()}
                    </span>
                    <span class="text-xs text-muted-foreground">{unit}</span>
                </div>
            </CardContent>
        </Card>
    }
}

#[component]
pub fn Dbpg005StatsCards() -> impl IntoView {
    let summary = use_dbpg005().summary;

    view! {
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard label=t_or("dbpg005.stats.provinces", "จังหวัด (Provinces)") unit=t_or("dbpg005.stats.unit_province", "จังหวัด") value=Signal::derive(move || summary.get().provinces) />
            <StatCard label=t_or("dbpg005.stats.districts", "อำเภอ / เขต (Districts)") unit=t_or("dbpg005.stats.unit_place", "แห่ง") value=Signal::derive(move || summary.get().districts) />
            <StatCard label=t_or("dbpg005.stats.sub_districts", "ตำบล / แขวง (Sub-districts)") unit=t_or("dbpg005.stats.unit_place", "แห่ง") value=Signal::derive(move || summary.get().sub_districts) />
            <StatCard label=t_or("dbpg005.stats.postal_codes", "รหัสไปรษณีย์ (Postal codes)") unit=t_or("dbpg005.stats.unit_item", "รายการ") value=Signal::derive(move || summary.get().postal_codes) />
        </div>
    }
}
