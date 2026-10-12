//! SUPG001 tabs (systems / modules) + system filter for the module list

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Select, SelectOption,
};

use crate::pages::su::supg001::models::UnitKind;
use crate::pages::su::supg001::providers::use_supg001;

#[component]
pub fn Supg001FilterBar() -> impl IntoView {
    let ctx = use_supg001();
    let tab_variant = move |k: UnitKind| {
        if ctx.tab.get() == k { ButtonVariant::Secondary } else { ButtonVariant::Ghost }
    };

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4 space-y-3">
                <div class="flex items-center rounded-lg border border-border/60 bg-muted/30 p-0.5 text-xs w-fit">
                    {move || UnitKind::ALL.into_iter().map(|k| view! {
                        <Button
                            variant=tab_variant(k)
                            size=ButtonSize::Sm
                            class="h-7 text-xs px-3"
                            on_click=Callback::new(move |_| ctx.set_tab.set(k))
                        >
                            {match k {
                                UnitKind::System => t_fmt("supg001.tab.system", "ระบบ ({count})", &[("count", &ctx.systems.get().len().to_string())]),
                                UnitKind::Module => t_fmt("supg001.tab.module", "โมดูล ({count})", &[("count", &ctx.modules.get().len().to_string())]),
                            }}
                        </Button>
                    }).collect_view()}
                </div>

                {move || (ctx.tab.get() == UnitKind::Module).then(|| view! {
                    <div class="max-w-md">
                        <Select
                            value=Signal::derive(move || ctx.system_filter.get())
                            on_change=Callback::new(move |v| ctx.set_system_filter.set(v))
                            class="bg-background text-sm font-medium"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("supg001.filter.all_systems", "🧩 ทุกระบบ (All Systems)")}</SelectOption>
                            {move || ctx.systems.get().into_iter().map(|s| {
                                let label = s.display_name();
                                view! { <SelectOption value=s.id.clone()>{label}</SelectOption> }
                            }).collect_view()}
                        </Select>
                    </div>
                })}
            </CardContent>
        </Card>
    }
}
