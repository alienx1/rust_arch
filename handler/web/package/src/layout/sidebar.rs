use leptos::prelude::*;
use tailwind_fuse::tw_merge;

use super::nav_icons::NavIcon;

#[component]
pub fn Sidebar(
    #[prop(into)] is_collapsed: Signal<bool>,
    /// Mobile (<md) off-canvas drawer state; ignored from `md` upward.
    #[prop(optional, into)]
    mobile_open: Signal<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    // Below `md` the sidebar is a fixed off-canvas drawer; from `md` it is a static column.
    let base = "fixed inset-y-0 left-0 z-40 flex h-dvh w-64 max-w-[85vw] flex-col border-r border-border bg-sidebar shadow-xl transition-all duration-300 select-none md:static md:z-30 md:h-screen md:max-w-none md:translate-x-0 md:shadow-none";
    let width_class = move || {
        if is_collapsed.get() { "md:w-16" } else { "md:w-64" }
    };
    let slide_class = move || {
        if mobile_open.get() { "translate-x-0" } else { "-translate-x-full" }
    };

    view! {
        <aside class=move || tw_merge!(base, width_class(), slide_class(), &class)>
            {children()}
        </aside>
    }
}

#[component]
pub fn SidebarHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex h-14 items-center border-b border-border px-4 gap-3";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarContent(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex-1 overflow-y-auto px-3 py-4 space-y-1 scrollbar-thin scrollbar-thumb-muted-foreground/20";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn SidebarGroup(
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "space-y-1";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {label.map(|lbl| {
                view! {
                    <h4 class="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">
                        {lbl}
                    </h4>
                }
            })}
            <ul class="space-y-1">
                {children()}
            </ul>
        </div>
    }
}

/// A titled group whose entries fold away behind a chevron; entries sit under a vertical guide
/// line. `has_active` opens it (and keeps it open) while one of its entries is the current page.
/// In the collapsed sidebar (from `md` up) the header and guide line go away and only the entries'
/// icons remain, one group under the other with a divider between groups.
#[component]
pub fn SidebarCollapsibleGroup(
    #[prop(into)] label: String,
    #[prop(into)] has_active: Signal<bool>,
    /// Icon name; empty = none
    #[prop(optional, into)] icon: String,
    #[prop(optional, into)] is_collapsed: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let (open, set_open) = signal(has_active.get_untracked());
    Effect::new(move |_| {
        if has_active.get() {
            set_open.set(true);
        }
    });
    let title = label.clone();

    view! {
        <div>
            <div class=move || if is_collapsed.get() { "hidden md:block mx-2 my-2 border-t border-border" } else { "hidden" }></div>
            <button
                type="button"
                title=title
                class=move || format!(
                    "flex w-full items-center justify-between gap-2 rounded-md px-3 py-2 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground {}",
                    if is_collapsed.get() { "md:hidden" } else { "" }
                )
                aria-expanded=move || open.get().to_string()
                on:click=move |_| set_open.update(|v| *v = !*v)
            >
                <span class="flex min-w-0 items-center gap-2.5">
                    {(!icon.is_empty()).then(|| view! { <NavIcon name=icon.clone() class="h-4 w-4 shrink-0 text-muted-foreground" /> })}
                    <span class="truncate">{label}</span>
                </span>
                <svg
                    class=move || if open.get() {
                        "h-4 w-4 shrink-0 text-muted-foreground transition-transform rotate-90"
                    } else {
                        "h-4 w-4 shrink-0 text-muted-foreground transition-transform"
                    }
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <path d="m9 18 6-6-6-6" />
                </svg>
            </button>
            <ul class=move || {
                let folded = !open.get();
                match (is_collapsed.get(), folded) {
                    (false, false) => "mx-4 mt-1 space-y-1 border-l border-border pl-2.5",
                    (false, true) => "mx-4 mt-1 space-y-1 border-l border-border pl-2.5 hidden",
                    (true, false) => "mx-4 mt-1 space-y-1 border-l border-border pl-2.5 md:mx-0 md:mt-0 md:border-0 md:pl-0",
                    // Folded group in the icon-only sidebar: its icons stay visible
                    (true, true) => "mx-4 mt-1 space-y-1 border-l border-border pl-2.5 hidden md:!block md:mx-0 md:mt-0 md:border-0 md:pl-0",
                }
            }>
                {children()}
            </ul>
        </div>
    }
}

#[component]
pub fn SidebarMenuItem(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(optional, into)] is_active: Signal<bool>,
    #[prop(optional, into)] badge: Option<String>,
    #[prop(optional, into)] icon: Option<ViewFn>,
    /// Icon by name (see `nav_icon_markup`); empty = none
    #[prop(optional, into)]
    icon_name: String,
    /// Icon-only presentation from `md` up (the collapsed sidebar); the label becomes a tooltip
    #[prop(optional, into)]
    is_collapsed: Signal<bool>,
    #[prop(optional, into)] class: String,
    /// Nested entry inside a collapsible group: soft highlight instead of the solid primary one
    #[prop(optional)]
    nested: bool,
) -> impl IntoView {
    let base = "group flex items-center justify-between rounded-md px-3 py-2 text-sm font-medium transition-colors cursor-pointer";
    let active_class = move || {
        if is_active.get() {
            if nested {
                "bg-accent text-accent-foreground"
            } else {
                "bg-primary text-primary-foreground shadow-sm"
            }
        } else {
            "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        }
    };
    let merged_class = move || {
        let collapsed = if is_collapsed.get() { "md:justify-center md:px-0" } else { "" };
        tw_merge!(base, active_class(), collapsed, &class)
    };
    let title = label.clone();
    let aria_current = move || if is_active.get() { "page" } else { "false" };

    view! {
        <li>
            <a href=href class=merged_class aria-current=aria_current title=title>
                <div class="flex items-center gap-3 truncate">
                    {icon.map(|ic| ic.run())}
                    {(!icon_name.is_empty()).then(|| view! { <NavIcon name=icon_name.clone() class="h-4 w-4 shrink-0" /> })}
                    <span class=move || if is_collapsed.get() { "truncate md:hidden" } else { "truncate" }>{label}</span>
                </div>
                {badge.map(|b| {
                    view! {
                        <span class=move || if is_collapsed.get() { "ml-auto flex md:hidden h-5 min-w-5 items-center justify-center rounded-full bg-secondary px-1.5 text-[10px] font-semibold text-secondary-foreground" } else { "ml-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-secondary px-1.5 text-[10px] font-semibold text-secondary-foreground" }>
                            {b}
                        </span>
                    }
                })}
            </a>
        </li>
    }
}

#[component]
pub fn SidebarFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "border-t border-border p-3 mt-auto";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
