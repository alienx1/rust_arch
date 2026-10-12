use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::MainLayout;

use crate::access::provide_access_context;
use crate::audit::use_audit_tracking;
use crate::layouts::menu_context::provide_menu_context;
use crate::pages::nt::ntpg000::components::NotificationBell;
use crate::pages::nt::ntpg000::providers::provide_notifications;

#[component]
pub fn MasterLayout() -> impl IntoView {
    // The sidebar comes from the database (SUPG002), not from a table in this crate
    // What the account may do per program: pages hide the buttons it cannot use
    provide_access_context();
    let menu = provide_menu_context();
    // Access audit: page views, clicks and processes of signed-in users
    use_audit_tracking();
    // The bell: notifications of the user, pushed to every page and tab
    provide_notifications();

    view! {
        <MainLayout
            nav_sections=menu.sections
            user_menu_items=menu.user_menu
            brand_name=t_or("common.brand.name", "Master Portal")
            brand_subtitle=t_or("common.brand.subtitle", "Enterprise v0.1")
            brand_initial="M"
            footer_label="Active-Active Spock pgEdge"
            breadcrumb_root=t_or("common.breadcrumb.root", "แพลตฟอร์ม")
            default_title=t_or("common.default.title", "Master Data Management Engine")
            topbar_actions=move || view! { <NotificationBell /> }
        />
    }
}
