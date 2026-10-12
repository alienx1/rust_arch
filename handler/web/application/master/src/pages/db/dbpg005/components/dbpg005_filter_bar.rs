//! DBPG005 level tabs + province / district filters (search, status chips, paging and the add
//! button live in the table header)

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Select, SelectOption,
};

use crate::pages::db::dbpg005::models::GeoLevel;
use crate::pages::db::dbpg005::providers::use_dbpg005;

#[component]
pub fn Dbpg005FilterBar() -> impl IntoView {
    let ctx = use_dbpg005();

    let tab_variant = move |level: GeoLevel| {
        if ctx.tab.get() == level { ButtonVariant::Secondary } else { ButtonVariant::Ghost }
    };
    // Province filter narrows districts, sub-districts and postal codes; district filter narrows
    // sub-districts and postal codes.
    let province_enabled = Signal::derive(move || ctx.tab.get() != GeoLevel::Province);
    let district_enabled = Signal::derive(move || {
        matches!(ctx.tab.get(), GeoLevel::SubDistrict | GeoLevel::PostalCode)
    });

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4 space-y-3">
                <div class="flex items-center rounded-lg border border-border/60 bg-muted/30 p-0.5 text-xs w-fit">
                    {move || GeoLevel::ALL.into_iter().map(|level| view! {
                        <Button
                            variant=tab_variant(level)
                            size=ButtonSize::Sm
                            class="h-7 text-xs px-3"
                            on_click=Callback::new(move |_| ctx.set_tab.run(level))
                        >
                            {level.label()}
                        </Button>
                    }).collect_view()}
                </div>

                {move || province_enabled.get().then(|| view! {
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 max-w-3xl">
                        <Select
                            value=Signal::derive(move || ctx.province_filter.get())
                            on_change=ctx.set_province
                            class="bg-background text-sm font-medium"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("dbpg005.filter.all_provinces", "🗺️ ทุกจังหวัด (All Provinces)")}</SelectOption>
                            {move || ctx.provinces.get().into_iter().map(|p| {
                                let label = p.display_name();
                                view! { <SelectOption value=p.id.clone()>{label}</SelectOption> }
                            }).collect_view()}
                        </Select>
                        {move || district_enabled.get().then(|| view! {
                            <Select
                                value=Signal::derive(move || ctx.district_filter.get())
                                on_change=ctx.set_district
                                class="bg-background text-sm font-medium"
                            >
                                <SelectOption value="ALL".to_string()>{t_or("dbpg005.filter.all_districts", "📍 ทุกอำเภอ (All Districts)")}</SelectOption>
                                {move || ctx.districts.get().into_iter().map(|d| {
                                    let label = d.display_name();
                                    view! { <SelectOption value=d.id.clone()>{label}</SelectOption> }
                                }).collect_view()}
                            </Select>
                        })}
                    </div>
                })}
            </CardContent>
        </Card>
    }
}
