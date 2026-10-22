//! The toast of a notification that just arrived over the stream: how a save, delete or report
//! went (green) or failed (red), on whatever page the user is on. Clicking it opens what it points
//! at; it goes away by itself.

use std::time::Duration;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::nt::ntpg000::providers::use_notifications;

/// How long a toast stays
const SHOW_FOR: Duration = Duration::from_secs(6);
/// A failure stays a little longer: it is the one the user has to act on
const SHOW_FAILURE_FOR: Duration = Duration::from_secs(10);

#[component]
pub fn NotificationToast() -> impl IntoView {
    let notifications = use_notifications();
    let navigate = use_navigate();
    // Each toast gets a number so an old timer never closes a newer toast
    let (shown, set_shown) = signal(0u64);

    Effect::new(move |_| {
        if let Some(n) = notifications.toast.get() {
            let this = shown.get_untracked() + 1;
            set_shown.set(this);
            let wait = if n.is_failure() { SHOW_FAILURE_FOR } else { SHOW_FOR };
            set_timeout(
                move || {
                    if shown.try_get_untracked() == Some(this) {
                        notifications.dismiss_toast.run(());
                    }
                },
                wait,
            );
        }
    });

    view! {
        <div class="fixed bottom-4 right-4 z-[60] w-[min(24rem,calc(100vw-2rem))] pointer-events-none" aria-live="polite">
            {move || notifications.toast.get().map(|n| {
                let failed = n.is_failure();
                let frame = if failed {
                    "border-destructive/50 bg-card"
                } else {
                    "border-emerald-500/50 bg-card"
                };
                let bar = if failed { "bg-destructive" } else { "bg-emerald-500" };
                let link = n.link.clone();
                let navigate = navigate.clone();
                view! {
                    <div
                        role="status"
                        class=format!("pointer-events-auto flex overflow-hidden rounded-lg border shadow-lg cursor-pointer {frame}")
                        on:click=move |_| {
                            notifications.dismiss_toast.run(());
                            if let Some(path) = &link {
                                navigate(path, Default::default());
                            }
                        }
                    >
                        <div class=format!("w-1.5 shrink-0 {bar}")></div>
                        <div class="flex-1 p-3 space-y-0.5">
                            <div class="text-sm font-semibold text-foreground">{n.title_text()}</div>
                            <div class="text-xs text-muted-foreground">{n.body_text()}</div>
                        </div>
                        <button
                            type="button"
                            class="px-2.5 text-muted-foreground hover:text-foreground cursor-pointer"
                            aria-label=t_or("common.action.close", "ปิด")
                            on:click=move |ev| {
                                ev.stop_propagation();
                                notifications.dismiss_toast.run(());
                            }
                        >
                            "×"
                        </button>
                    </div>
                }
            })}
        </div>
    }
}
