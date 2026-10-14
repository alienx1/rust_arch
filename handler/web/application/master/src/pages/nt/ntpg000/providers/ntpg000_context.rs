//! NTPG000 list context: all recent notifications of the user, reloaded when the server pushes.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::nt::ntpg000::models::NotificationDto;
use crate::pages::nt::ntpg000::providers::use_notifications;
use crate::pages::nt::ntpg000::services::Ntpg000Service;
use crate::routes::program_paths::NTPG000;

/// Notifications the list page loads: all of them (the server keeps at most 300 per user)
const PAGE_LIMIT: u64 = 300;

#[derive(Clone, Copy)]
pub struct Ntpg000Context {
    pub items: ReadSignal<Vec<NotificationDto>>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub refresh: Callback<()>,
    /// Marks the notification as read and opens what it points at
    pub open: Callback<NotificationDto>,
    pub mark_all_read: Callback<()>,
}

pub fn provide_ntpg000_context() -> Ntpg000Context {
    let service = Ntpg000Service::new();
    let notifications = use_notifications();
    let navigate = use_navigate();
    let (items, set_items) = signal(Vec::<NotificationDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new(move |_| {
        refresh_trigger.track();
        // A push (new or read elsewhere): show it
        notifications.changes.track();
        set_is_loading.set(true);
        let s = service.clone();
        spawn_local(async move {
            match s.fetch_list(PAGE_LIMIT).await {
                Ok(list) => {
                    set_items.set(list.items);
                    set_error.set(None);
                }
                Err(e) => set_error.set(Some(format!(
                    "{}: {e}",
                    t_or("ntpg000.error.load", "ไม่สามารถโหลดการแจ้งเตือนได้")
                ))),
            }
            set_is_loading.set(false);
        });
    });

    let ctx = Ntpg000Context {
        items,
        is_loading,
        error,
        refresh: Callback::new(move |_| set_refresh_trigger.update(|n| *n += 1)),
        open: Callback::new(move |n: NotificationDto| {
            if !n.read {
                notifications.mark_read.run(n.id.clone());
            }
            navigate(n.link.as_deref().unwrap_or(NTPG000), Default::default());
        }),
        mark_all_read: Callback::new(move |_| notifications.mark_all_read.run(())),
    };
    provide_context(ctx);
    ctx
}

pub fn use_ntpg000() -> Ntpg000Context {
    expect_context::<Ntpg000Context>()
}
