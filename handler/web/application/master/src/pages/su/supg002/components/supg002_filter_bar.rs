//! SUPG002 system / module filters (search, status chips, paging and the add button live in the
//! program table header)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, Select, SelectOption};

use crate::pages::su::supg002::providers::use_supg002;

#[component]
pub fn Supg002FilterBar() -> impl IntoView {
    let ctx = use_supg002();

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 max-w-3xl">
                    <Select
                        value=Signal::derive(move || ctx.system_filter.get())
                        on_change=ctx.set_system
                        class="bg-background text-sm font-medium"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("supg002.filter.all_systems", "🧩 ทุกระบบ (All Systems)")}</SelectOption>
                        {move || ctx.options.get().systems.into_iter().map(|s| {
                            let label = s.display_name();
                            view! { <SelectOption value=s.id.clone()>{label}</SelectOption> }
                        }).collect_view()}
                    </Select>
                    <Select
                        value=Signal::derive(move || ctx.module_filter.get())
                        on_change=ctx.set_module
                        class="bg-background text-sm font-medium"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("supg002.filter.all_modules", "📦 ทุกโมดูล (All Modules)")}</SelectOption>
                        {move || {
                            let system = ctx.system_filter.get();
                            ctx.options
                                .get()
                                .modules
                                .into_iter()
                                .filter(|m| system == "ALL" || m.system_id == system)
                                .map(|m| {
                                    let label = m.display_name();
                                    view! { <SelectOption value=m.id.clone()>{label}</SelectOption> }
                                })
                                .collect_view()
                        }}
                    </Select>
                </div>
            </CardContent>
        </Card>
    }
}
