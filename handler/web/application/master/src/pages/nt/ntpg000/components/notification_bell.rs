//! Bell in the topbar: a badge with the number of unread notifications and a dropdown with the
//! latest ones. Works on every page; the list and the count come from the server, so every tab
//! and device of the user agrees.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;
use web_package::ui::component::dismiss_on_outside;

use crate::pages::al::alpg000::components::audit_time_format::EventTime;
use crate::pages::nt::ntpg000::models::NotificationDto;
use crate::pages::nt::ntpg000::providers::use_notifications;
use crate::routes::program_paths::NTPG000;

const BELL_PATH: &str = "M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0";

/// Dot colour of a notification: red when it reports a failure
pub fn dot_class(kind: &str) -> &'static str {
    if kind.ends_with("_failed") { "bg-red-500" } else { "bg-emerald-500" }
}

/// `2026-10-12 08:51:38` in the browser's time zone
pub fn local_time(rfc3339: &str) -> String {
    EventTime::parse(rfc3339).map(|t| format!("{} {}", t.date, t.time)).unwrap_or_else(|| rfc3339.to_string())
}

#[component]
pub fn NotificationBell() -> impl IntoView {
    let notifications = use_notifications();
    let navigate = use_navigate();
    let (open, set_open) = signal(false);

    // Closing the panel reads nothing: a notification is read when its row is clicked (or by the
    // explicit "mark all read" button)
    let close = move || set_open.set(false);
    let root = NodeRef::<leptos::html::Div>::new();
    dismiss_on_outside(root, move || open.get_untracked(), close);
    let navigate_all = navigate.clone();
    let view_all = Callback::new(move |()| {
        close();
        navigate_all(NTPG000, Default::default());
    });
    let open_item = Callback::new(move |n: NotificationDto| {
        set_open.set(false);
        if !n.read {
            notifications.mark_read.run(n.id.clone());
        }
        navigate(n.link.as_deref().unwrap_or(NTPG000), Default::default());
    });

    view! {
        <div class="relative" node_ref=root>
            <button
                type="button"
                class="relative p-1.5 rounded-md hover:bg-muted text-muted-foreground transition-colors cursor-pointer"
                aria-label=move || t_or("ntpg000.bell.label", "การแจ้งเตือน")
                title=move || t_or("ntpg000.bell.label", "การแจ้งเตือน")
                on:click=move |_| if open.get_untracked() { close() } else { set_open.set(true) }
            >
                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d=BELL_PATH />
                </svg>
                {move || {
                    let n = notifications.unread.get();
                    (n > 0).then(|| view! {
                        <span class="absolute -top-0.5 -right-0.5 min-w-[16px] h-4 px-1 rounded-full bg-destructive text-destructive-foreground text-[10px] font-semibold leading-4 text-center">
                            {if n > 9 { "9+".to_string() } else { n.to_string() }}
                        </span>
                    })
                }}
            </button>
            <Show when=move || open.get()>
                <div class="absolute right-0 z-50 mt-2 w-[360px] max-w-[calc(100vw-2rem)] rounded-md border border-border bg-background shadow-lg">
                    <div class="flex items-center justify-between px-3 py-2 border-b border-border">
                        <span class="text-sm font-semibold">{t_or("ntpg000.bell.title", "การแจ้งเตือน")}</span>
                        <button
                            type="button"
                            class="text-xs text-primary hover:underline cursor-pointer"
                            on:click=move |_| notifications.mark_all_read.run(())
                        >
                            {t_or("ntpg000.bell.mark_read", "อ่านทั้งหมดแล้ว")}
                        </button>
                    </div>
                    <div class="max-h-[60vh] overflow-y-auto">
                        {move || {
                            let items = notifications.items.get();
                            if items.is_empty() {
                                return view! {
                                    <div class="px-3 py-6 text-center text-xs text-muted-foreground">
                                        {t_or("ntpg000.bell.empty", "ยังไม่มีการแจ้งเตือน")}
                                    </div>
                                }.into_any();
                            }
                            items.into_iter().map(|n| view! { <BellItem n=n on_open=open_item /> }).collect_view().into_any()
                        }}
                    </div>
                    <button
                        type="button"
                        class="w-full px-3 py-2 border-t border-border text-xs text-primary hover:bg-muted/50 cursor-pointer"
                        on:click=move |_| view_all.run(())
                    >
                        {t_or("ntpg000.bell.view_all", "ดูการแจ้งเตือนทั้งหมด")}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn BellItem(n: NotificationDto, on_open: Callback<NotificationDto>) -> impl IntoView {
    let unread = !n.read;
    let (dot, time) = (dot_class(&n.kind), local_time(&n.created_at));
    let (title, body) = (n.title.clone(), n.body.clone());
    view! {
        <button
            type="button"
            // Unread: a primary accent bar on the left and a tinted background (`bg-primary/N` has no
            // alpha in this Tailwind setup, so the tint is a color-mix of the theme colour); read: same layout,
            // transparent bar, so the rows do not shift
            class=format!(
                "flex w-full items-start gap-2 px-3 py-2 text-left cursor-pointer border-b border-border/40 border-l-4 {}",
                if unread {
                    "border-l-primary bg-[color-mix(in_oklch,var(--primary)_12%,transparent)] hover:bg-[color-mix(in_oklch,var(--primary)_20%,transparent)]"
                } else {
                    "border-l-transparent hover:bg-muted/50"
                }
            )
            on:click=move |_| on_open.run(n.clone())
        >
            <span class=format!("mt-1.5 h-2 w-2 shrink-0 rounded-full {dot}")></span>
            <span class="min-w-0 flex-1">
                <span class="flex items-center justify-between gap-2">
                    <span class=format!("text-xs {}", if unread { "font-bold text-foreground" } else { "font-medium text-muted-foreground" })>{title}</span>
                    <span class="text-[10px] text-muted-foreground whitespace-nowrap">{time}</span>
                </span>
                <span class=format!("block text-[11px] break-all {}", if unread { "text-foreground/80" } else { "text-muted-foreground" })>{body}</span>
            </span>
        </button>
    }
}
