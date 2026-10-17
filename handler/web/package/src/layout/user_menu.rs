//! Signed-in user's dropdown menu: avatar + name open a panel with the user's own pages (e.g.
//! "my reports", notifications) and the sign-out action.

use crate::ui::component::{Avatar, dismiss_on_outside};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// One entry of the user menu
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserMenuItem {
    pub href: String,
    pub label: String,
    /// SVG path data (24x24, stroke icon) drawn before the label
    pub icon_path: String,
}

impl UserMenuItem {
    pub fn new(href: impl Into<String>, label: impl Into<String>, icon_path: impl Into<String>) -> Self {
        Self { href: href.into(), label: label.into(), icon_path: icon_path.into() }
    }
}

const ITEM_CLASS: &str = "flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-sm transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none";
const LOGOUT_ICON: &str = "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4 M16 17l5-5-5-5 M21 12H9";
const CHEVRON_ICON: &str = "M6 9l6 6 6-6";

#[component]
pub fn UserMenu(
    /// Display name (or email)
    #[prop(into)]
    name: String,
    /// Second line under the name in the panel header (e.g. the role or email)
    #[prop(optional, into)]
    subtitle: String,
    /// Shown under the name in the panel header (falls back to `subtitle`)
    #[prop(optional, into)]
    email: String,
    /// Photo shown in the avatar (the initial when `None`)
    #[prop(optional, into)]
    image_url: Signal<Option<String>>,
    #[prop(into)] items: Signal<Vec<UserMenuItem>>,
    #[prop(into)] logout_label: String,
    on_logout: Callback<()>,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let (open, set_open) = signal(false);
    let root = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside(root, move || open.get_untracked(), move || set_open.set(false));
    let initial = name.chars().next().unwrap_or('U').to_uppercase().to_string();
    let header_name = name.clone();
    let header_initial = header_name.chars().next().unwrap_or('U').to_uppercase().to_string();
    let header_subtitle = if email.is_empty() { subtitle.clone() } else { email };

    view! {
        <div class="relative" node_ref=root>
            <button
                type="button"
                class="flex items-center gap-2 rounded-md p-1 pr-1.5 transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring cursor-pointer"
                aria-haspopup="menu"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| set_open.update(|o| *o = !*o)
            >
                {move || match image_url.get() {
                    Some(url) => view! { <Avatar src=url fallback=initial.clone() class="h-8 w-8" /> }.into_any(),
                    None => view! { <Avatar fallback=initial.clone() class="h-8 w-8" /> }.into_any(),
                }}
                <div class="hidden sm:flex flex-col text-left">
                    <span class="text-xs font-semibold text-foreground truncate max-w-[140px]">{name}</span>
                    <span class="text-[10px] text-muted-foreground">{subtitle}</span>
                </div>
                <svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path d=CHEVRON_ICON />
                </svg>
            </button>
            <Show when=move || open.get()>
                <div
                    class="absolute right-0 z-50 mt-2 w-60 overflow-hidden rounded-lg border border-border bg-popover p-1.5 text-popover-foreground shadow-lg"
                    role="menu"
                >
                    <div class="flex items-center gap-2.5 px-2.5 py-2">
                        {let fb = header_initial.clone(); move || match image_url.get() {
                            Some(url) => view! { <Avatar src=url fallback=fb.clone() class="h-9 w-9" /> }.into_any(),
                            None => view! { <Avatar fallback=fb.clone() class="h-9 w-9" /> }.into_any(),
                        }}
                        <div class="min-w-0">
                            <div class="truncate text-sm font-semibold">{header_name.clone()}</div>
                            <div class="truncate text-xs text-muted-foreground">{header_subtitle.clone()}</div>
                        </div>
                    </div>
                    <div class="my-1 h-px bg-border"></div>
                    {move || {
                        items
                            .get()
                            .into_iter()
                            .map(|item| {
                                let href = item.href.clone();
                                let active = move || pathname.get() == href;
                                view! {
                                    <A
                                        href=item.href.clone()
                                        attr:class=move || format!("{ITEM_CLASS} {}", if active() { "bg-accent font-semibold text-primary" } else { "" })
                                        attr:role="menuitem"
                                        on:click=move |_| set_open.set(false)
                                    >
                                        <svg class="h-4 w-4 shrink-0 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d=item.icon_path.clone() />
                                        </svg>
                                        <span class="truncate">{item.label.clone()}</span>
                                    </A>
                                }
                            })
                            .collect_view()
                    }}
                    <div class="my-1 h-px bg-border"></div>
                    <button
                        type="button"
                        class=format!("{ITEM_CLASS} text-destructive")
                        role="menuitem"
                        on:click=move |_| {
                            set_open.set(false);
                            on_logout.run(());
                        }
                    >
                        <svg class="h-4 w-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d=LOGOUT_ICON />
                        </svg>
                        <span>{logout_label.clone()}</span>
                    </button>
                </div>
            </Show>
        </div>
    }
}
