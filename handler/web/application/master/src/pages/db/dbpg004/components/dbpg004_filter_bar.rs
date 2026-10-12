//! DBPG004 category filter (search, status chips, paging and the add button live in the lookup
//! table header)

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{Card, CardContent, Select, SelectOption};

use crate::pages::db::dbpg004::providers::use_dbpg004;

#[component]
pub fn Dbpg004FilterBar() -> impl IntoView {
    let ctx = use_dbpg004();

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4 flex flex-col sm:flex-row sm:items-center gap-2.5 max-w-3xl">
                <label class="text-xs font-semibold text-muted-foreground whitespace-nowrap">
                    {t_or("dbpg004.filter.select_category", "เลือกหมวดหมู่:")}
                </label>
                <div class="flex-1">
                    <Select
                        value=Signal::derive(move || ctx.category_filter.get())
                        on_change=ctx.set_category
                        class="bg-background text-sm font-medium"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("dbpg004.filter.all_categories", "📚 ทุกหมวดหมู่ (All Categories)")}</SelectOption>
                        {move || ctx.categories.get().into_iter().map(|c| {
                            let label = t_fmt(
                                "dbpg004.filter.category_option",
                                "{category}{lock} • {count} รายการ",
                                &[
                                    ("category", c.category.as_str()),
                                    ("lock", if c.is_protected { " 🔒" } else { "" }),
                                    ("count", &c.total.to_string()),
                                ],
                            );
                            view! { <SelectOption value=c.category.clone()>{label}</SelectOption> }
                        }).collect_view()}
                    </Select>
                </div>
            </CardContent>
        </Card>
    }
}
