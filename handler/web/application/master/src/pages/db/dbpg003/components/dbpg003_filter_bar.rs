//! DBPG003 company + branch filters (search, status chips, paging and the add button live in the
//! employee table header)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, Select, SelectOption};

use crate::pages::db::dbpg003::providers::use_dbpg003;

#[component]
pub fn Dbpg003FilterBar() -> impl IntoView {
    let ctx = use_dbpg003();

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 max-w-3xl">
                    <Select
                        value=Signal::derive(move || ctx.company_filter.get())
                        on_change=ctx.set_company
                        class="bg-background text-sm font-medium"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("dbpg003.filter.all_companies", "🏬 ทุกบริษัท (All Companies)")}</SelectOption>
                        {move || ctx.companies.get().into_iter().map(|c| {
                            let label = c.display_name();
                            view! { <SelectOption value=c.id.clone()>{label}</SelectOption> }
                        }).collect_view()}
                    </Select>
                    <Select
                        value=Signal::derive(move || ctx.branch_filter.get())
                        on_change=ctx.set_branch
                        class="bg-background text-sm font-medium"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("dbpg003.filter.all_branches", "🏢 ทุกสาขา (All Branches)")}</SelectOption>
                        {move || {
                            let company = ctx.company_filter.get();
                            ctx.branches
                                .get()
                                .into_iter()
                                .filter(|b| company == "ALL" || b.company_id == company)
                                .map(|b| {
                                    let label = b.display_name();
                                    view! { <SelectOption value=b.id.clone()>{label}</SelectOption> }
                                })
                                .collect_view()
                        }}
                    </Select>
                </div>
            </CardContent>
        </Card>
    }
}
