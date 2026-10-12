//! Dashboard Reactive Context & State Provider

use crate::pages::dashboard::models::DashboardSummary;
use crate::pages::dashboard::services::DashboardService;
use leptos::prelude::*;
use web_package::core::i18n::t_fmt;

/// Shared reactive state context for Dashboard components.
#[derive(Clone, Copy)]
pub struct DashboardContext {
    pub data: ReadSignal<Option<DashboardSummary>>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub refresh: Callback<()>,
}

/// Provides DashboardContext to the Leptos reactive tree.
pub fn provide_dashboard_context() -> DashboardContext {
    let (data, set_data) = signal(Option::<DashboardSummary>::None);
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    let service = DashboardService::new();

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        set_is_loading.set(true);
        set_error.set(None);
        let svc = service.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match svc.fetch_summary().await {
                Ok(summary) => {
                    set_data.set(Some(summary));
                }
                Err(e) => {
                    set_error.set(Some(t_fmt("sypg000.error.api_failed", "เชื่อมต่อ API ล้มเหลว: {error}", &[("error", &e.to_string())])));
                }
            }
            set_is_loading.set(false);
        });
    });

    let refresh = Callback::new(move |()| {
        set_refresh_trigger.update(|n| *n += 1);
    });

    let ctx = DashboardContext {
        data,
        is_loading,
        error,
        refresh,
    };

    provide_context(ctx);
    ctx
}

/// Access the DashboardContext within the component subtree.
pub fn use_dashboard() -> DashboardContext {
    expect_context::<DashboardContext>()
}
