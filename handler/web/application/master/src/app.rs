//! Root Application Component for Master Data SPA
//!
//! Wires auth context + `Router`; the route tree lives in `routes::MasterRoutes`.

use crate::routes::MasterRoutes;
use leptos::prelude::*;
use leptos_router::components::*;
use web_package::core::auth::provide_auth_context;
use web_package::core::i18n::provide_i18n_context;
use web_package::ui::component::Spinner;

#[component]
pub fn App() -> impl IntoView {
    // 1. Initialize Ory Kratos Authentication Context
    let kratos_url = option_env!("WEB_KRATOS_URL")
        .or(option_env!("KRATOS_PUBLIC_URL"))
        .unwrap_or("/api/kratos");
    provide_auth_context(kratos_url);

    // 2. Initialize i18n Context (fetches dictionary from db_label)
    let i18n = provide_i18n_context();

    view! {
        <Router>
            <main class="min-h-screen bg-background text-foreground font-sans antialiased">
                // Pages read their texts once while building, so the whole route tree is
                // re-mounted when the language (dictionary) changes
                {move || if i18n.is_loading.get() {
                    view! {
                        <div class="min-h-screen flex items-center justify-center">
                            <Spinner class="h-6 w-6 text-primary" />
                        </div>
                    }.into_any()
                } else {
                    i18n.dict.track();
                    view! { <MasterRoutes /> }.into_any()
                }}
            </main>
        </Router>
    }
}
