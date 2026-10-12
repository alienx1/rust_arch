//! DBPG002 KPI Stats Cards (branches & departments)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent};

use crate::pages::db::dbpg002::providers::use_dbpg002;

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
pub fn Dbpg002StatsCards() -> impl IntoView {
    let summary = use_dbpg002().summary;

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard
                label=t_or("dbpg002.stats.total_branches", "สาขาทั้งหมด (Branches)")
                unit=t_or("dbpg002.stats.unit_branch", "สาขา")
                value=Signal::derive(move || summary.get().total_branches)
            />
            <StatCard
                label=t_or("dbpg002.stats.active_branches", "สาขาที่เปิดใช้งาน")
                unit=t_or("dbpg002.stats.unit_branch", "สาขา")
                value=Signal::derive(move || summary.get().active_branches)
                value_class="text-emerald-600 dark:text-emerald-400"
            />
            <StatCard
                label=t_or("dbpg002.stats.total_departments", "แผนกทั้งหมด (Departments)")
                unit=t_or("dbpg002.stats.unit_department", "แผนก")
                value=Signal::derive(move || summary.get().total_departments)
            />
            <StatCard
                label=t_or("dbpg002.stats.active_departments", "แผนกที่เปิดใช้งาน")
                unit=t_or("dbpg002.stats.unit_department", "แผนก")
                value=Signal::derive(move || summary.get().active_departments)
                value_class="text-emerald-600 dark:text-emerald-400"
            />
        </div>
    }
}
