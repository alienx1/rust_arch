use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or, use_i18n};
use web_package::layout::{BrandOverride, MainLayout};

use crate::access::provide_access_context;
use crate::audit::use_audit_tracking;
use crate::layouts::company_switcher::CompanySwitcher;
use crate::layouts::menu_context::provide_menu_context;
use crate::pages::nt::ntpg000::components::{NotificationBell, NotificationToast};
use crate::pages::nt::ntpg000::providers::provide_notifications;

#[component]
pub fn MasterLayout() -> impl IntoView {
    // The sidebar comes from the database (SUPG002), not from a table in this crate
    // What the account may do per program: pages hide the buttons it cannot use
    let access = provide_access_context();
    let menu = provide_menu_context();
    let i18n = use_i18n();
    // Access audit: page views, clicks and processes of signed-in users
    use_audit_tracking();
    // The bell: notifications of the user, pushed to every page and tab
    provide_notifications();

    view! {
        // Toast of a notification that just arrived (how a save went), over every page
        <NotificationToast />
        <MainLayout
            nav_sections=menu.sections
            user_menu_items=menu.user_menu
            user_image_url=access.image_url
            brand=Signal::derive(move || {
                // The signed-in user's organization (name + logo) and company (subtitle)
                // `localized` reads the saved language, which is not a signal: track the change here
                let _ = i18n.current_lang.get();
                let org = access.organization.get()?;
                let name = localized(&org.name_th, &org.name_en);
                (!name.is_empty()).then(|| BrandOverride {
                    name,
                    // The company being looked at: the one picked in the switcher, "all companies"
                    // for a headquarters overview, else the account's own company
                    subtitle: match access.selected_company_id.get() {
                        Some(id) => access.companies.get().iter().find(|c| c.id == id).cloned(),
                        None if access.unlimited.get() => None,
                        None => access.company.get(),
                    }
                    .map(|c| localized(&c.name_th, &c.name_en))
                    .or_else(|| access.unlimited.get().then(|| t_or("common.company.all", "ทุกบริษัท (ภาพรวม)")))
                    .filter(|s| !s.is_empty()),
                    image_url: org.image_url,
                })
            })
            brand_name=t_or("common.brand.name", "Master Portal")
            brand_subtitle=t_or("common.brand.subtitle", "Enterprise v0.1")
            brand_initial="M"
            footer_label="Active-Active Spock pgEdge"
            breadcrumb_root=t_or("common.breadcrumb.root", "แพลตฟอร์ม")
            default_title=t_or("common.default.title", "Master Data Management Engine")
            brand_subtitle_slot=move || view! { <CompanySwitcher /> }
            topbar_actions=move || view! { <NotificationBell /> }
        />
    }
}
