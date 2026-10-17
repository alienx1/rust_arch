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

/// Organization or company of the account's employee
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OwnUnitDto {
    #[serde(default, deserialize_with = "crate::pages::db::dbpg000::models::deserialize_id")]
    pub id: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name_th: String,
    #[serde(default)]
    pub name_en: String,
    /// Logo URL (`None` = no logo)
    #[serde(default)]
    pub image_url: Option<String>,
}

/// A branch of the switcher
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OwnBranchDto {
    #[serde(default, deserialize_with = "crate::pages::db::dbpg000::models::deserialize_id")]
    pub id: String,
    #[serde(default, deserialize_with = "crate::pages::db::dbpg000::models::deserialize_id")]
    pub company_id: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name_th: String,
    #[serde(default)]
    pub name_en: String,
}

/// `GET /me` answer
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AccessMeDto {
    #[serde(default)]
    pub username: Option<String>,
    /// Photo of the employee behind the account (topbar avatar)
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub organization: Option<OwnUnitDto>,
    #[serde(default)]
    pub company: Option<OwnUnitDto>,
    /// Headquarters account: may see the whole organization
    #[serde(default)]
    pub is_hq: bool,
    /// Sees the whole organization (headquarters, or a group assigned at the organization)
    #[serde(default)]
    pub unlimited: bool,
    /// Companies the switcher offers
    #[serde(default)]
    pub companies: Vec<OwnUnitDto>,
    /// The company the server limits this request to (`None` = all companies)
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub selected_company_id: Option<String>,
    /// Branches the switcher offers
    #[serde(default)]
    pub branches: Vec<OwnBranchDto>,
    /// The branch the server limits this request to (`None` = all branches)
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub selected_branch_id: Option<String>,
    /// Program code -> granted actions
    #[serde(default)]
    pub permissions: HashMap<String, Vec<String>>,
}

#[derive(Clone, Copy)]
pub struct AccessContext {
    permissions: ReadSignal<Option<HashMap<String, Vec<String>>>>,
    /// Photo URL of the signed-in account's employee (`None` = no photo)
    pub image_url: ReadSignal<Option<String>>,
    /// Organization and company of the account's employee: the sidebar brand
    pub organization: ReadSignal<Option<OwnUnitDto>>,
    pub company: ReadSignal<Option<OwnUnitDto>>,
    /// Sees the whole organization: "all" in the switchers means everything
    pub unlimited: ReadSignal<bool>,
    /// Companies the topbar switcher offers
    pub companies: ReadSignal<Vec<OwnUnitDto>>,
    /// The company the app is looking at (`None` = all companies)
    pub selected_company_id: ReadSignal<Option<String>>,
    /// Branches the switcher offers
    pub branches: ReadSignal<Vec<OwnBranchDto>>,
    /// The branch the app is looking at (`None` = all branches)
    pub selected_branch_id: ReadSignal<Option<String>>,
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
    let (image_url, set_image_url) = signal(Option::<String>::None);
    let (organization, set_organization) = signal(Option::<OwnUnitDto>::None);
    let (company, set_company) = signal(Option::<OwnUnitDto>::None);
    let (unlimited, set_unlimited) = signal(false);
    let (companies, set_companies) = signal(Vec::<OwnUnitDto>::new());
    let (selected_company_id, set_selected_company_id) = signal(Option::<String>::None);
    let (branches, set_branches) = signal(Vec::<OwnBranchDto>::new());
    let (selected_branch_id, set_selected_branch_id) = signal(Option::<String>::None);
    let (version, set_version) = signal(0u32);

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            set_permissions.set(None);
            set_image_url.set(None);
            set_organization.set(None);
            set_company.set(None);
            set_unlimited.set(false);
            set_companies.set(Vec::new());
            set_selected_company_id.set(None);
            set_branches.set(Vec::new());
            set_selected_branch_id.set(None);
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
            if organization.get_untracked() != me.organization {
                set_organization.set(me.organization.clone());
            }
            if unlimited.get_untracked() != me.unlimited {
                set_unlimited.set(me.unlimited);
            }
            if companies.get_untracked() != me.companies {
                set_companies.set(me.companies.clone());
            }
            if branches.get_untracked() != me.branches {
                set_branches.set(me.branches.clone());
            }
            if selected_branch_id.get_untracked() != me.selected_branch_id {
                set_selected_branch_id.set(me.selected_branch_id.clone());
            }
            if selected_company_id.get_untracked() != me.selected_company_id {
                set_selected_company_id.set(me.selected_company_id.clone());
            }
            if company.get_untracked() != me.company {
                set_company.set(me.company.clone());
            }
            if image_url.get_untracked() != me.image_url {
                set_image_url.set(me.image_url.clone());
            }
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

    let ctx = AccessContext {
        permissions,
        image_url,
        organization,
        company,
        unlimited,
        companies,
        selected_company_id,
        branches,
        selected_branch_id,
        version,
    };
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

/// An optional id that arrives as a UUID (16 bytes in CBOR, a string in JSON) or null
fn deserialize_opt_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Id(#[serde(deserialize_with = "crate::pages::db::dbpg000::models::deserialize_id")] String);
    Ok(Option::<Id>::deserialize(deserializer)?.map(|id| id.0))
}
