//! Organization Selector and Action Toolbar Component

use web_package::core::i18n::{t_fmt, t_or};
use leptos::prelude::*;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Select, SelectOption,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg000::providers::use_dbpg000;

#[component]
pub fn OrgSelectorBar() -> impl IntoView {
    let ctx = use_dbpg000();
    let can_write = can("DBPG001", Action::Write);
    let orgs = ctx.organizations;
    let selected_org_id = ctx.selected_org_id;

    let on_select_org = move |val: String| {
        ctx.set_selected_org_id.set(val);
    };

    let on_add_org = move |_| {
        ctx.open_create_org.run(());
    };

    let on_edit_org = move |_| {
        let current_id = selected_org_id.get();
        if current_id != "ALL" && !current_id.is_empty() {
            if let Some(org) = orgs.get().into_iter().find(|o| o.id == current_id) {
                ctx.open_edit_org.run(org);
            }
        }
    };

    let on_add_company = move |_| {
        ctx.open_create_company.run(());
    };

    let is_org_selected = Signal::derive(move || {
        let id = selected_org_id.get();
        id != "ALL" && !id.is_empty()
    });

    view! {
        <Card class="border-border/60 bg-card/60 backdrop-blur-sm shadow-sm">
            <CardContent class="p-4 flex flex-col md:flex-row md:items-center justify-between gap-4">
                // Left: Organization Dropdown & Edit button
                <div class="flex flex-col sm:flex-row sm:items-center gap-2.5 flex-1 max-w-2xl">
                    <label class="text-xs font-semibold text-muted-foreground whitespace-nowrap flex items-center gap-1.5">
                        <svg class="h-4 w-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M3 21h18M3 7v14M21 7v14M6 10h2M6 14h2M6 18h2M16 10h2M16 14h2M16 18h2M10 21V3h4v18" />
                        </svg>
                        {t_or("dbpg000.selector.label", "เลือกกลุ่มองค์กร:")}
                    </label>

                    <div class="flex-1">
                        <Select
                            value=selected_org_id
                            on_change=Callback::new(on_select_org)
                            class="bg-background text-sm font-medium"
                        >
                            <SelectOption value="ALL".to_string()>
                                {t_or("dbpg000.selector.all", "🏢 ทั้งหมด (All Organizations)")}
                            </SelectOption>
                            {move || {
                                orgs.get().into_iter().map(|org| {
                                    let label = org.display_name();
                                    view! {
                                        <SelectOption value=org.id.clone()>
                                            {t_fmt("dbpg000.selector.org_option", "{label} • ({count} บริษัท)", &[("label", &label), ("count", &org.company_count.to_string())])}
                                        </SelectOption>
                                    }
                                }).collect_view()
                            }}
                        </Select>
                    </div>

                    // Edit Org button (visible when a specific org is chosen)
                    <Show when=move || is_org_selected.get()>
                        <Button
                            variant=ButtonVariant::Outline
                            size=ButtonSize::Sm
                            class="text-xs h-9 gap-1.5 border-border hover:bg-accent shrink-0"
                            on_click=Callback::new(on_edit_org)
                        >
                            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                            </svg>
                            {move || if can_write.get() { t_or("dbpg000.action.edit_org", "แก้ไข Org") } else { t_or("common.action.view", "ดู") }}
                        </Button>
                    </Show>
                </div>

                // Right: Add Buttons
                <div class="flex items-center gap-2 justify-end">
                    <PermissionGate program="DBPG001" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        class="text-xs h-9 gap-1.5 border-primary/30 text-primary hover:bg-primary/10"
                        on_click=Callback::new(on_add_org)
                    >
                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <line x1="12" y1="5" x2="12" y2="19" />
                            <line x1="5" y1="12" x2="19" y2="12" />
                        </svg>
                        {t_or("dbpg000.action.add_org", "เพิ่มองค์กร (Org)")}
                    </Button>

                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        class="text-xs h-9 gap-1.5 shadow-sm"
                        on_click=Callback::new(on_add_company)
                    >
                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <line x1="12" y1="5" x2="12" y2="19" />
                            <line x1="5" y1="12" x2="19" y2="12" />
                        </svg>
                        {t_or("dbpg000.action.add_company", "เพิ่มบริษัท (Company)")}
                    </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
