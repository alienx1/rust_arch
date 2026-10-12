//! Main Sidebar with router-aware active menu highlighting (like Angular `routerLinkActive`)

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use super::main_navigation::NavSection;
use super::sidebar::{
    Sidebar, SidebarCollapsibleGroup, SidebarContent, SidebarFooter, SidebarHeader, SidebarMenuItem,
};

#[component]
pub fn MainSidebar(
    #[prop(into)] is_collapsed: Signal<bool>,
    #[prop(into)] mobile_open: Signal<bool>,
    #[prop(into)] nav_sections: Signal<Vec<NavSection>>,
    #[prop(into)] brand_name: String,
    #[prop(default = None)] brand_subtitle: Option<String>,
    #[prop(default = None)] brand_initial: Option<String>,
    #[prop(default = None)] footer_label: Option<String>,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let initial = brand_initial.unwrap_or_else(|| brand_name.chars().take(1).collect());

    view! {
        <Sidebar is_collapsed=is_collapsed mobile_open=mobile_open>
            <SidebarHeader>
                <a href="/" class="flex items-center gap-2.5 font-bold text-foreground">
                    <div class="h-8 w-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-black text-sm shadow-sm ring-1 ring-primary/20">
                        {initial}
                    </div>
                    <div class=move || if is_collapsed.get() { "flex flex-col md:hidden" } else { "flex flex-col" }>
                        <span class="text-sm font-semibold tracking-tight">{brand_name.clone()}</span>
                        {brand_subtitle.clone().map(|sub| view! {
                            <span class="text-[10px] text-muted-foreground font-mono">{sub}</span>
                        })}
                    </div>
                </a>
            </SidebarHeader>

            <SidebarContent>
                {move || nav_sections.get().into_iter().map(|section| {
                    let entries = section.items.clone();
                    let has_active = Signal::derive(move || {
                        let path = pathname.get();
                        entries.iter().any(|item| item.is_active_for(&path))
                    });
                    view! {
                        <SidebarCollapsibleGroup label=section.label icon=section.icon.clone().unwrap_or_default() is_collapsed=is_collapsed has_active=has_active>
                            {section.items.into_iter().map(|item| {
                                let entry = item.clone();
                                let is_active = Signal::derive(move || entry.is_active_for(&pathname.get()));
                                match item.badge {
                                    Some(badge) => view! {
                                        <SidebarMenuItem href=item.href label=item.label badge=badge icon_name=item.icon.clone().unwrap_or_default() is_collapsed=is_collapsed is_active=is_active nested=true />
                                    }.into_any(),
                                    None => view! {
                                        <SidebarMenuItem href=item.href label=item.label icon_name=item.icon.clone().unwrap_or_default() is_collapsed=is_collapsed is_active=is_active nested=true />
                                    }.into_any(),
                                }
                            }).collect_view()}
                        </SidebarCollapsibleGroup>
                    }
                }).collect_view()}
            </SidebarContent>

            {footer_label.map(|label| view! {
                <SidebarFooter>
                    <div class="flex items-center justify-between text-xs text-muted-foreground">
                        <span class="flex items-center gap-2">
                            <span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
                            <span class=move || if is_collapsed.get() { "md:hidden" } else { "" }>{label.clone()}</span>
                        </span>
                    </div>
                </SidebarFooter>
            })}
        </Sidebar>
    }
}
