//! Switches the tracker on for signed-in users and records the route changes.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use web_package::core::auth::use_auth;

use crate::audit::{audit_click_listener, audit_tracker};

/// Call once inside the router, in the layout that only signed-in users reach.
pub fn use_audit_tracking() {
    let auth = use_auth();
    let location = use_location();

    audit_tracker::start();
    audit_click_listener::install();

    Effect::new(move |_| {
        audit_tracker::set_enabled(auth.is_authenticated.get());
    });
    // Runs after the enabling effect above (effects run in creation order)
    Effect::new(move |_| {
        let path = location.pathname.get();
        if auth.is_authenticated.get() {
            audit_tracker::track_page_view(&path);
        }
    });
}
