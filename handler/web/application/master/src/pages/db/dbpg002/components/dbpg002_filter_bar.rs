//! DBPG002 toolbar: company filter, Branches/Departments tabs and add button (search and status
//! chips live in the table headers)

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Select, SelectOption,
};

use crate::access::{Action, PermissionGate};
use crate::pages::db::dbpg002::providers::{Dbpg002Tab, use_dbpg002};

#[component]
pub fn Dbpg002FilterBar() -> impl IntoView {
    let ctx = use_dbpg002();

    let tab_variant = move |t: Dbpg002Tab| {
        if ctx.tab.get() == t { ButtonVariant::Secondary } else { ButtonVariant::Ghost }
    };

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4 space-y-3">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-3">
                    <div class="flex flex-col sm:flex-row sm:items-center gap-2.5 flex-1 max-w-2xl">
                        <label class="text-xs font-semibold text-muted-foreground whitespace-nowrap">
                            {t_or("dbpg002.filter.company_label", "เลือกบริษัท:")}
                        </label>
                        <div class="flex-1">
                            <Select
                                value=ctx.company_filter
                                on_change=Callback::new(move |v| ctx.set_company_filter.set(v))
                                class="bg-background text-sm font-medium"
                            >
                                <SelectOption value="ALL".to_string()>
                                    {t_or("dbpg002.filter.all_companies", "🏬 ทั้งหมด (All Companies)")}
                                </SelectOption>
                                {move || {
                                    ctx.companies.get().into_iter().map(|c| {
                                        let label = c.display_name();
                                        view! { <SelectOption value=c.id.clone()>{label}</SelectOption> }
                                    }).collect_view()
                                }}
                            </Select>
                        </div>
                    </div>

                    <PermissionGate program="DBPG002" action=Action::Write>
                        <Button
                            variant=ButtonVariant::Default
                            size=ButtonSize::Sm
                            class="text-xs"
                            on_click=Callback::new(move |_| ctx.open_create.run(()))
                        >
                            {move || match ctx.tab.get() {
                                Dbpg002Tab::Branches => t_or("dbpg002.filter.add_branch", "+ เพิ่มสาขา"),
                                Dbpg002Tab::Departments => t_or("dbpg002.filter.add_department", "+ เพิ่มแผนก"),
                            }}
                        </Button>
                    </PermissionGate>
                </div>

                <div class="flex items-center rounded-lg border border-border/60 bg-muted/30 p-0.5 text-xs w-fit">
                    {move || view! {
                        <Button
                            variant=tab_variant(Dbpg002Tab::Branches)
                            size=ButtonSize::Sm
                            class="h-7 text-xs px-3"
                            on_click=Callback::new(move |_| ctx.set_tab.set(Dbpg002Tab::Branches))
                        >
                            {move || t_fmt("dbpg002.filter.tab_branches", "สาขา ({count})", &[("count", &ctx.branches.get().len().to_string())])}
                        </Button>
                        <Button
                            variant=tab_variant(Dbpg002Tab::Departments)
                            size=ButtonSize::Sm
                            class="h-7 text-xs px-3"
                            on_click=Callback::new(move |_| ctx.set_tab.set(Dbpg002Tab::Departments))
                        >
                            {move || t_fmt("dbpg002.filter.tab_departments", "แผนก ({count})", &[("count", &ctx.departments.get().len().to_string())])}
                        </Button>
                    }}
                </div>
            </CardContent>
        </Card>
    }
}
