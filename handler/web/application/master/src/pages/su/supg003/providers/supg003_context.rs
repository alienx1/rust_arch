//! SUPG003 list context: permission groups with search and status filters (client-side).
//!
//! Create / edit / matrix actions navigate to the SUPG003 pages.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg003::models::PermissionGroupDto;
use crate::pages::su::supg003::services::Supg003Service;
use crate::routes::{supg003_group_path, supg003_matrix_path};

/// Shared reactive state for the SUPG003 list page
#[derive(Clone, Copy)]
pub struct Supg003Context {
    pub groups: ReadSignal<Vec<PermissionGroupDto>>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_search_query: WriteSignal<String>,
    pub set_status_filter: WriteSignal<String>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<PermissionGroupDto>,
    pub open_matrix: Callback<PermissionGroupDto>,
}

/// Provides Supg003Context into the reactive tree
pub fn provide_supg003_context() -> Supg003Context {
    let service = Supg003Service::new();
    let (groups, set_groups) = signal(Vec::<PermissionGroupDto>::new());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        set_is_loading.set(true);
        set_error.set(None);
        let s = service.clone();
        spawn_local(async move {
            match s.fetch_groups().await {
                Ok(v) => set_groups.set(v),
                Err(e) => {
                    set_groups.set(Vec::new());
                    set_error.set(Some(e));
                }
            }
            set_is_loading.set(false);
        });
    });

    let refresh = Callback::new(move |()| set_refresh_trigger.update(|v| *v = v.wrapping_add(1)));

    let navigate = use_navigate();
    let open_create = Callback::new({
        let navigate = navigate.clone();
        move |()| navigate(&supg003_group_path(None), Default::default())
    });
    let open_edit = Callback::new({
        let navigate = navigate.clone();
        move |g: PermissionGroupDto| navigate(&supg003_group_path(Some(&g.id)), Default::default())
    });
    let open_matrix = Callback::new(move |g: PermissionGroupDto| {
        navigate(&supg003_matrix_path(&g.id), Default::default())
    });

    let ctx = Supg003Context {
        groups,
        search_query,
        status_filter,
        is_loading,
        error,
        set_search_query,
        set_status_filter,
        refresh,
        open_create,
        open_edit,
        open_matrix,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Supg003Context
pub fn use_supg003() -> Supg003Context {
    use_context::<Supg003Context>()
        .expect("Supg003Context must be provided by provide_supg003_context()")
}
