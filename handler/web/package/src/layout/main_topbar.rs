//! Main Topbar with route-derived breadcrumb and authenticated user menu

use super::header::{Header, HeaderLeft, HeaderRight};
use crate::core::auth::use_auth;
use crate::core::i18n::{LanguageSwitcher, t_or};
use crate::ui::component::Spinner;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

use super::main_navigation::{NavSection, nav_label_for};
use super::theme_toggle::ThemeToggle;
use super::user_menu::{UserMenu, UserMenuItem};

#[component]
pub fn MainTopbar(
    on_toggle_sidebar: Callback<()>,
    on_toggle_mobile_menu: Callback<()>,
    #[prop(into)] nav_sections: Signal<Vec<NavSection>>,
    #[prop(default = None)] breadcrumb_root: Option<String>,
    #[prop(default = None)] default_title: Option<String>,
    #[prop(default = None)] login_path: Option<String>,
    /// Extra controls before the language switcher (e.g. a notification bell)
    #[prop(optional, into)] actions: ViewFn,
    /// Entries of the user dropdown above "sign out" (e.g. my reports, notifications)
    #[prop(optional, into)] user_menu_items: Signal<Vec<UserMenuItem>>,
    /// Photo of the signed-in user for the avatar
    #[prop(optional, into)] user_image_url: Signal<Option<String>>,
) -> impl IntoView {
    let auth = use_auth();
    let pathname = use_location().pathname;
    let default_title = default_title.unwrap_or_default();
    let login_path = login_path.unwrap_or_else(|| "/login".to_string());
    let title = move || {
        nav_label_for(&nav_sections.get(), &pathname.get()).unwrap_or_else(|| default_title.clone())
    };

    view! {
        <Header>
            <HeaderLeft>
                <button
                    type="button"
                    class="p-1.5 rounded-md hover:bg-muted text-muted-foreground transition-colors cursor-pointer md:hidden"
                    on:click=move |_| on_toggle_mobile_menu.run(())
                    title="Menu"
                    aria-label="Open menu"
                >
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <line x1="3" y1="12" x2="21" y2="12"></line>
                        <line x1="3" y1="6" x2="21" y2="6"></line>
                        <line x1="3" y1="18" x2="21" y2="18"></line>
                    </svg>
                </button>
                <button
                    type="button"
                    class="hidden md:block p-1.5 rounded-md hover:bg-muted text-muted-foreground transition-colors cursor-pointer"
                    on:click=move |_| on_toggle_sidebar.run(())
                    title="Toggle Sidebar"
                >
                    <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <line x1="3" y1="12" x2="21" y2="12"></line>
                        <line x1="3" y1="6" x2="21" y2="6"></line>
                        <line x1="3" y1="18" x2="21" y2="18"></line>
                    </svg>
                </button>
                <div class="flex min-w-0 items-center gap-2 text-sm font-medium text-foreground">
                    {breadcrumb_root.map(|root| view! {
                        <span class="text-muted-foreground hidden sm:inline">{root}</span>
                        <span class="text-muted-foreground hidden sm:inline">"/"</span>
                    })}
                    <span class="font-semibold text-foreground truncate">{title}</span>
                </div>
            </HeaderLeft>

            <HeaderRight>
                <div class="flex items-center gap-2 sm:gap-3">
                    {actions.run()}
                    <ThemeToggle />
                    <LanguageSwitcher />
                    {move || {
                        if auth.is_loading.get() {
                            view! { <Spinner class="h-4 w-4 text-primary" /> }.into_any()
                        } else if let Some(user) = auth.user.get() {
                            let anon = t_or("common.user.anonymous", "ผู้ใช้งาน");
                            let display_name = user.name().or_else(|| user.email()).unwrap_or(anon);
                            let email = user.email().unwrap_or_default();
                            view! {
                                <UserMenu
                                    name=display_name
                                    email=email
                                    subtitle="Administrator"
                                    items=user_menu_items
                                    image_url=user_image_url
                                    logout_label=t_or("common.action.logout", "ออกจากระบบ")
                                    on_logout=Callback::new(move |_| auth.logout())
                                />
                            }.into_any()
                        } else {
                            let login_text = move || t_or("common.action.login", "เข้าสู่ระบบ");
                            view! {
                                <a href=login_path.clone() class="text-xs text-primary hover:underline font-medium">
                                    {login_text}
                                </a>
                            }.into_any()
                        }
                    }}
                </div>
            </HeaderRight>
        </Header>
    }
}
