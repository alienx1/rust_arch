//! SUPG001 list context: systems and modules, system filter, search, status.
//!
//! Both lists are small, so they are loaded whole and the tables page them client-side.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg001::models::{ModuleDto, SystemDto, UnitKind};
use crate::pages::su::supg001::services::Supg001Service;
use crate::routes::supg001_form_path;

/// Shared reactive state for the SUPG001 list page
#[derive(Clone, Copy)]
pub struct Supg001Context {
    pub systems: ReadSignal<Vec<SystemDto>>,
    pub modules: ReadSignal<Vec<ModuleDto>>,
    pub tab: ReadSignal<UnitKind>,
    pub system_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_tab: WriteSignal<UnitKind>,
    pub set_system_filter: WriteSignal<String>,
    pub set_search_query: WriteSignal<String>,
    pub set_status_filter: WriteSignal<String>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit_system: Callback<SystemDto>,
    pub open_edit_module: Callback<ModuleDto>,
}

/// Provides Supg001Context into the reactive tree
pub fn provide_supg001_context() -> Supg001Context {
    let service = Supg001Service::new();
    let (systems, set_systems) = signal(Vec::<SystemDto>::new());
    let (modules, set_modules) = signal(Vec::<ModuleDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (tab, set_tab) = signal(UnitKind::System);
    let (system_filter, set_system_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let system = system_filter.get();
        set_is_loading.set(true);
        set_error.set(None);
        let s = service.clone();
        spawn_local(async move {
            let mut first_err: Option<String> = None;
            match s.fetch_systems().await {
                Ok(v) => set_systems.set(v),
                Err(e) => first_err = Some(e),
            }
            let filter = (system != "ALL").then_some(system);
            match s.fetch_modules(filter).await {
                Ok(v) => set_modules.set(v),
                Err(e) => {
                    set_modules.set(Vec::new());
                    first_err = first_err.or(Some(e));
                }
            }
            if let Some(e) = first_err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    });

    let refresh = Callback::new(move |()| set_refresh_trigger.update(|v| *v = v.wrapping_add(1)));

    let navigate = use_navigate();
    let open_create = Callback::new({
        let navigate = navigate.clone();
        move |()| {
            let kind = tab.get_untracked();
            let path = supg001_form_path(kind, None);
            let system = system_filter.get_untracked();
            if kind == UnitKind::Module && system != "ALL" {
                navigate(&format!("{path}?system_id={system}"), Default::default());
            } else {
                navigate(&path, Default::default());
            }
        }
    });
    let open_edit_system = Callback::new({
        let navigate = navigate.clone();
        move |s: SystemDto| navigate(&supg001_form_path(UnitKind::System, Some(&s.id)), Default::default())
    });
    let open_edit_module = Callback::new(move |m: ModuleDto| {
        navigate(&supg001_form_path(UnitKind::Module, Some(&m.id)), Default::default())
    });

    let ctx = Supg001Context {
        systems,
        modules,
        tab,
        system_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_tab,
        set_system_filter,
        set_search_query,
        set_status_filter,
        refresh,
        open_create,
        open_edit_system,
        open_edit_module,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Supg001Context
pub fn use_supg001() -> Supg001Context {
    use_context::<Supg001Context>()
        .expect("Supg001Context must be provided by provide_supg001_context()")
}
