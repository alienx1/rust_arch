//! Main Authenticated Layout (persistent shell + `<Outlet/>`, like Angular `<router-outlet>`)
//!
//! The sidebar, topbar and collapse state stay mounted across navigations;
//! only the routed page inside `<Outlet/>` is swapped.

use super::shell::AppShell;
use crate::core::auth::use_auth;
use crate::ui::component::Spinner;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::Outlet;
use leptos_router::hooks::{use_location, use_navigate};

use super::main_navigation::NavSection;
use super::main_sidebar::{BrandOverride, MainSidebar};
use super::main_topbar::MainTopbar;
use super::user_menu::UserMenuItem;

#[component]
pub fn MainLayout(
    /// Application menu (also drives the topbar breadcrumb title); reactive, so it can be
    /// loaded from an API after the layout has mounted.
    #[prop(into)]
    nav_sections: Signal<Vec<NavSection>>,
    #[prop(into)] brand_name: String,
    #[prop(optional, into)] brand_subtitle: Option<String>,
    #[prop(optional, into)] brand_initial: Option<String>,
    #[prop(optional, into)] footer_label: Option<String>,
    #[prop(optional, into)] breadcrumb_root: Option<String>,
    /// Topbar title when the current path is not in `nav_sections`.
    #[prop(optional, into)]
    default_title: Option<String>,
    /// Redirect target of the auth guard (default `/login`).
    #[prop(optional, into)]
    login_path: Option<String>,
    /// Extra controls at the right of the topbar (e.g. a notification bell).
    #[prop(optional, into)]
    topbar_actions: Option<ViewFn>,
    /// Entries of the user dropdown in the topbar (above "sign out").
    #[prop(optional, into)]
    user_menu_items: Signal<Vec<UserMenuItem>>,
    /// Photo of the signed-in user for the topbar avatar.
    #[prop(optional, into)]
    user_image_url: Signal<Option<String>>,
    /// Replaces the sidebar brand (name, subtitle, logo) while it is `Some`.
    #[prop(optional, into)]
    brand: Signal<Option<BrandOverride>>,
    /// A control shown instead of the line under the brand name (e.g. the company switcher).
    #[prop(optional, into)]
    brand_subtitle_slot: Option<ViewFn>,
) -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let (is_collapsed, set_is_collapsed) = signal(false);
    // Mobile (<md) off-canvas sidebar; closes automatically on every route change.
    let (mobile_open, set_mobile_open) = signal(false);
    let pathname = use_location().pathname;
    Effect::new(move |_| {
        pathname.track();
        set_mobile_open.set(false);
    });
    let login_path = login_path.unwrap_or_else(|| "/login".to_string());
    let guard_login_path = login_path.clone();

    // Auth guard (like Angular `CanActivate`): client-side redirect, no page reload
    Effect::new(move |_| {
        if !auth.is_loading.get() && !auth.is_authenticated.get() {
            navigate(
                &guard_login_path,
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    // Active session heartbeat: verify session validity with Ory Kratos every 45s.
    // The loop stops when the layout is disposed so remounts never stack loops.
    let heartbeat_auth = auth;
    let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    {
        let alive = alive.clone();
        on_cleanup(move || alive.store(false, std::sync::atomic::Ordering::Relaxed));
    }
    wasm_bindgen_futures::spawn_local(async move {
        while alive.load(std::sync::atomic::Ordering::Relaxed) {
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                if let Some(win) = web_sys::window() {
                    let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 45_000);
                }
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

            if alive.load(std::sync::atomic::Ordering::Relaxed)
                && heartbeat_auth.is_authenticated.get_untracked()
            {
                heartbeat_auth.check_session_silent();
            }
        }
    });

    // Re-verify immediately when the user returns to this tab/window (removed on cleanup).
    let focus_auth = auth;
    let focus_handle = window_event_listener(leptos::ev::focus, move |_| {
        if focus_auth.is_authenticated.get_untracked() {
            focus_auth.check_session_silent();
        }
    });
    on_cleanup(move || focus_handle.remove());

    let on_toggle_sidebar = Callback::new(move |_| set_is_collapsed.update(|c| *c = !*c));
    let on_toggle_mobile_menu = Callback::new(move |_| set_mobile_open.update(|o| *o = !*o));

    view! {
        <AppShell
            sidebar=move || view! {
                <Show when=move || mobile_open.get()>
                    <div
                        class="fixed inset-0 z-30 bg-black/40 backdrop-blur-sm lg:hidden"
                        on:click=move |_| set_mobile_open.set(false)
                    ></div>
                </Show>
                <MainSidebar
                    is_collapsed=is_collapsed
                    mobile_open=mobile_open
                    nav_sections=nav_sections
                    brand_name=brand_name.clone()
                    brand_subtitle=brand_subtitle.clone()
                    brand_initial=brand_initial.clone()
                    footer_label=footer_label.clone()
                    brand=brand
                    use_subtitle_slot=brand_subtitle_slot.is_some()
                    brand_subtitle_slot=brand_subtitle_slot.clone().unwrap_or_else(|| ViewFn::from(|| ()))
                />
            }
            header=move || {
                let topbar_actions = topbar_actions.clone().unwrap_or_else(|| ViewFn::from(|| ()));
                view! {
                    <MainTopbar
                        on_toggle_sidebar=on_toggle_sidebar
                        on_toggle_mobile_menu=on_toggle_mobile_menu
                        nav_sections=nav_sections
                        breadcrumb_root=breadcrumb_root.clone()
                        default_title=default_title.clone()
                        login_path=Some(login_path.clone())
                        actions=topbar_actions
                        user_menu_items=user_menu_items
                        user_image_url=user_image_url
                    />
                }
            }
        >
            <Show
                when=move || !auth.is_loading.get()
                fallback=|| view! {
                    <div class="h-full flex items-center justify-center">
                        <Spinner class="h-6 w-6 text-primary" />
                    </div>
                }
            >
                <Outlet />
            </Show>
        </AppShell>
    }
}
