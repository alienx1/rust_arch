//! Access context: loads `GET /api/v1/access/me` once the user is signed in and answers
//! "may I do `action` on `program`?" (reactive: reading it inside a view closure tracks it).
//!
//! Like the session, the rights are re-checked in the background (every 45 s and when the tab
//! regains focus), so a change made in SUPG003 reaches an open page without a reload. The
//! sidebar menu follows the same `version` tick.
//!
//! Until the answer has arrived (and when it cannot be loaded) nothing is allowed, so buttons
//! appear only once their permission is known.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_package::core::auth::use_auth;

use crate::api_client::ApiClient;

/// API base path of the access endpoint
pub const ACCESS_API: &str = "/api/v1/access";

/// What an account can do with a program
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// See / search (GET)
    Read,
    /// Create / change (POST, PUT)
    Write,
    /// Delete and other approvals
    Approve,
}

impl Action {
    fn code(self) -> &'static str {
        match self {
            Action::Read => "READ",
            Action::Write => "WRITE",
            Action::Approve => "APPROVE",
        }
    }
}

/// `GET /me` answer
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AccessMeDto {
    #[serde(default)]
    pub username: Option<String>,
    /// Program code -> granted actions
    #[serde(default)]
    pub permissions: HashMap<String, Vec<String>>,
}

#[derive(Clone, Copy)]
pub struct AccessContext {
    permissions: ReadSignal<Option<HashMap<String, Vec<String>>>>,
    /// Bumped on every background re-check; whatever depends on the rights (the menu) tracks it
    pub version: ReadSignal<u32>,
}

/// Seconds between background re-checks (the session heartbeat uses the same period)
const REFRESH_MS: i32 = 45_000;

impl AccessContext {
    /// Whether `action` is granted on `program` (false until the rights are loaded)
    pub fn can(&self, program: &str, action: Action) -> bool {
        self.permissions.with(|p| {
            p.as_ref()
                .and_then(|m| m.get(program))
                .is_some_and(|actions| actions.iter().any(|a| a == action.code()))
        })
    }
}

/// Provides the access context; reloads when the signed-in state changes and re-checks in the
/// background while signed in
pub fn provide_access_context() -> AccessContext {
    let auth = use_auth();
    let (permissions, set_permissions) = signal(Option::<HashMap<String, Vec<String>>>::None);
    let (version, set_version) = signal(0u32);

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            set_permissions.set(None);
            return;
        }
        let _ = version.get(); // background re-check
        spawn_local(async move {
            let api = ApiClient::new(ACCESS_API);
            // A failed re-check keeps what is known; only the first load falls back to "nothing"
            let Ok(me) = api.get::<AccessMeDto>("/me", "สิทธิ์การใช้งาน").await else {
                if permissions.get_untracked().is_none() {
                    set_permissions.set(Some(HashMap::new()));
                }
                return;
            };
            // Write only on a real change, so open pages are not re-rendered every 45 s
            if permissions.get_untracked().as_ref() != Some(&me.permissions) {
                set_permissions.set(Some(me.permissions));
            }
        });
    });

    // Heartbeat: stops when the layout is disposed so remounts never stack loops
    let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    {
        let alive = alive.clone();
        on_cleanup(move || alive.store(false, std::sync::atomic::Ordering::Relaxed));
    }
    spawn_local(async move {
        while alive.load(std::sync::atomic::Ordering::Relaxed) {
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                if let Some(win) = web_sys::window() {
                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, REFRESH_MS);
                }
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            if alive.load(std::sync::atomic::Ordering::Relaxed) && auth.is_authenticated.get_untracked() {
                set_version.update(|v| *v = v.wrapping_add(1));
            }
        }
    });

    // Re-check right away when the user comes back to this tab / window
    let focus_handle = window_event_listener(leptos::ev::focus, move |_| {
        if auth.is_authenticated.get_untracked() {
            set_version.update(|v| *v = v.wrapping_add(1));
        }
    });
    on_cleanup(move || focus_handle.remove());

    let ctx = AccessContext { permissions, version };
    provide_context(ctx);
    ctx
}

pub fn use_access() -> AccessContext {
    expect_context::<AccessContext>()
}

/// Reactive "may I?" for use in view closures: `Show when=can("DBPG004", Action::Write)`
pub fn can(program: &'static str, action: Action) -> Signal<bool> {
    let ctx = use_access();
    Signal::derive(move || ctx.can(program, action))
}
