//! DBPG003 KPI Stats Cards (employees)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent};

use crate::pages::db::dbpg003::providers::use_dbpg003;

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
pub fn Dbpg003StatsCards() -> impl IntoView {
    let summary = use_dbpg003().summary;

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <StatCard
                label=t_or("dbpg003.stats.total", "พนักงานทั้งหมด (Employees)")
                unit=t_or("dbpg003.stats.unit_person", "คน")
                value=Signal::derive(move || summary.get().total_employees)
            />
            <StatCard
                label=t_or("dbpg003.stats.active", "กำลังปฏิบัติงาน (Active)")
                unit=t_or("dbpg003.stats.unit_person", "คน")
                value=Signal::derive(move || summary.get().active_employees)
                value_class="text-emerald-600 dark:text-emerald-400"
            />
            <StatCard
                label=t_or("dbpg003.stats.inactive", "พ้นสภาพ/ปิดใช้งาน (Inactive)")
                unit=t_or("dbpg003.stats.unit_person", "คน")
                value=Signal::derive(move || {
                    let s = summary.get();
                    s.total_employees.saturating_sub(s.active_employees)
                })
                value_class="text-amber-600 dark:text-amber-400"
            />
            <StatCard
                label=t_or("dbpg003.stats.with_account", "มีบัญชีผู้ใช้ระบบ (User Account)")
                unit=t_or("dbpg003.stats.unit_person", "คน")
                value=Signal::derive(move || summary.get().with_user_account)
            />
        </div>
    }
}
