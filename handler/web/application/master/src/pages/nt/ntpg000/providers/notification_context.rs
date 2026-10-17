//! The bell: the user's recent notifications and how many are unread, from the notification
//! service. Opens the push stream once the session is known; every push (a new notification, or
//! a read elsewhere) reloads the list, so every tab and device of the user stays right. Provided
//! by the layout, so it works on every page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use web_package::core::auth::use_auth;

use crate::pages::nt::ntpg000::models::NotificationDto;
use crate::pages::nt::ntpg000::services::{
    NotificationStream, Ntpg000Service, play_chime, save_sound_enabled, sound_enabled, unlock_audio_on_first_gesture,
};

/// Notifications the bell lists
pub const BELL_ITEMS: u64 = 10;

#[derive(Clone, Copy)]
pub struct Notifications {
    /// Newest first, at most [`BELL_ITEMS`]
    pub items: ReadSignal<Vec<NotificationDto>>,
    /// Unread in all
    pub unread: ReadSignal<u64>,
    /// Counts every reload caused by a push: pages watch it to refresh themselves
    pub changes: ReadSignal<u64>,
    /// The user wants a chime for a new notification (kept in the browser)
    pub sound_on: ReadSignal<bool>,
    pub toggle_sound: Callback<()>,
    /// Marks one as read (server side), then reloads
    pub mark_read: Callback<String>,
    /// Marks everything as read (server side), then reloads
    pub mark_all_read: Callback<()>,
}

pub fn provide_notifications() -> Notifications {
    let auth = use_auth();
    let service = Ntpg000Service::new();
    let (items, set_items) = signal(Vec::<NotificationDto>::new());
    let (unread, set_unread) = signal(0u64);
    let (changes, set_changes) = signal(0u64);
    let (sound_on, set_sound_on) = signal(sound_enabled());
    let stream = StoredValue::new_local(Option::<NotificationStream>::None);
    unlock_audio_on_first_gesture();

    let reload = {
        let service = service.clone();
        move || {
            let s = service.clone();
            spawn_local(async move {
                if let Ok(list) = s.fetch_list(BELL_ITEMS).await {
                    set_items.set(list.items);
                    set_unread.set(list.unread);
                }
            });
        }
    };

    // Open once the user is signed in (this also loads what arrived while the tab was closed);
    // the stream is closed with the layout
    {
        let reload = reload.clone();
        Effect::new(move |_| {
            if auth.is_authenticated.get() && stream.with_value(Option::is_none) {
                reload();
                let on_push = reload.clone();
                stream.set_value(Ntpg000Service::open_stream(
                    move || {
                        set_changes.update(|c| *c += 1);
                        on_push();
                    },
                    move || {
                        if sound_on.get_untracked() {
                            play_chime();
                        }
                    },
                ));
            }
        });
    }
    on_cleanup(move || {
        stream.try_set_value(None);
    });

    let ctx = Notifications {
        items,
        unread,
        changes,
        sound_on,
        toggle_sound: Callback::new(move |()| {
            let on = !sound_on.get_untracked();
            set_sound_on.set(on);
            save_sound_enabled(on);
            if on {
                // The click that switches it on is also a preview of what it sounds like
                play_chime();
            }
        }),
        mark_read: {
            let (service, reload) = (service.clone(), reload.clone());
            Callback::new(move |id: String| {
                let (s, reload) = (service.clone(), reload.clone());
                spawn_local(async move {
                    let _ = s.mark_read(&id).await;
                    reload();
                });
            })
        },
        mark_all_read: Callback::new(move |_| {
            let (s, reload) = (service.clone(), reload.clone());
            spawn_local(async move {
                let _ = s.mark_all_read().await;
                reload();
            });
        }),
    };
    provide_context(ctx);
    ctx
}

pub fn use_notifications() -> Notifications {
    expect_context::<Notifications>()
}
