//! Main Sidebar with router-aware active menu highlighting (like Angular `routerLinkActive`)

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use super::main_navigation::NavSection;
use super::sidebar::{
    Sidebar, SidebarCollapsibleGroup, SidebarContent, SidebarFooter, SidebarHeader, SidebarMenuItem,
};

/// What the brand block shows instead of the fixed app brand, e.g. the signed-in user's organization
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandOverride {
    pub name: String,
    pub subtitle: Option<String>,
    /// Logo shown instead of the initial
    pub image_url: Option<String>,
}

#[component]
pub fn MainSidebar(
    #[prop(into)] is_collapsed: Signal<bool>,
    #[prop(into)] mobile_open: Signal<bool>,
    #[prop(into)] nav_sections: Signal<Vec<NavSection>>,
    #[prop(into)] brand_name: String,
    #[prop(default = None)] brand_subtitle: Option<String>,
    #[prop(default = None)] brand_initial: Option<String>,
    #[prop(default = None)] footer_label: Option<String>,
    /// Replaces the name, subtitle and logo above while it is `Some`
    #[prop(optional, into)] brand: Signal<Option<BrandOverride>>,
    /// A control shown instead of the line under the brand name (e.g. the company switcher)
    #[prop(optional, into)] brand_subtitle_slot: ViewFn,
    /// Whether `brand_subtitle_slot` is used
    #[prop(optional)] use_subtitle_slot: bool,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let initial = brand_initial.unwrap_or_else(|| brand_name.chars().take(1).collect());

    view! {
        <Sidebar is_collapsed=is_collapsed mobile_open=mobile_open>
            <SidebarHeader>
                // The logo and the name link home; the line under the name may be a control (not
                // inside a link, or a click on it would navigate)
                <div class="flex items-center gap-2.5 font-bold text-foreground">
                    {
                        let (name, subtitle, initial, subtitle_slot) = (brand_name.clone(), brand_subtitle.clone(), initial.clone(), brand_subtitle_slot.clone());
                        move || {
                            let current = brand.get();
                            let logo = current.as_ref().and_then(|b| b.image_url.clone());
                            let shown_name = current.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| name.clone());
                            let shown_subtitle = match &current {
                                Some(b) => b.subtitle.clone(),
                                None => subtitle.clone(),
                            };
                            let shown_initial = current
                                .as_ref()
                                .map(|b| b.name.chars().take(1).collect::<String>())
                                .unwrap_or_else(|| initial.clone());
                            let subtitle_line = match use_subtitle_slot {
                                true => Some(subtitle_slot.run()),
                                false => shown_subtitle.map(|sub| view! {
                                    <span class="text-[10px] text-muted-foreground font-mono truncate">{sub}</span>
                                }.into_any()),
                            };
                            view! {
                                <a href="/" class="shrink-0">
                                    {match logo {
                                        Some(url) => view! {
                                            <img src=url alt="" class="h-8 w-8 shrink-0 rounded-lg object-contain ring-1 ring-border bg-background" />
                                        }.into_any(),
                                        None => view! {
                                            <div class="h-8 w-8 shrink-0 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-black text-sm shadow-sm ring-1 ring-primary/20">
                                                {shown_initial}
                                            </div>
                                        }.into_any(),
                                    }}
                                </a>
                                <div class=move || if is_collapsed.get() { "flex flex-col min-w-0 flex-1 md:hidden" } else { "flex flex-col min-w-0 flex-1" }>
                                    <a href="/" class="text-sm font-semibold tracking-tight truncate">{shown_name}</a>
                                    {subtitle_line}
                                </div>
                            }
                        }
                    }
                </div>
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
