//! SUPG002 list context: system / module filters, search, status, paging of the program list.
//!
//! Create/edit actions navigate to the SUPG002 form page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg002::models::{ProgramDto, ProgramOptionsDto, ProgramPageDto};
use crate::pages::su::supg002::services::{ProgramListQuery, Supg002Service};
use crate::routes::supg002_program_path;

/// Rows per page of the program list
pub const PAGE_SIZE: u64 = 25;

/// Shared reactive state for the SUPG002 list page
#[derive(Clone, Copy)]
pub struct Supg002Context {
    pub options: ReadSignal<ProgramOptionsDto>,
    pub page: ReadSignal<ProgramPageDto>,
    pub system_filter: ReadSignal<String>,
    pub module_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Changing a filter returns to page 1; a system change clears the module filter
    pub set_system: Callback<String>,
    pub set_module: Callback<String>,
    pub set_search: Callback<String>,
    pub set_status: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<ProgramDto>,
}

/// Provides Supg002Context into the reactive tree
pub fn provide_supg002_context() -> Supg002Context {
    let service = Supg002Service::new();
    let (options, set_options) = signal(ProgramOptionsDto::default());
    let (page, set_page_data) = signal(ProgramPageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (system_filter, set_system_filter) = signal("ALL".to_string());
    let (module_filter, set_module_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let query = ProgramListQuery {
            system_id: Some(system_filter.get()),
            module_id: Some(module_filter.get()),
            search: Some(search_query.get()),
            active: match status_filter.get().as_str() {
                "ACTIVE" => Some(true),
                "INACTIVE" => Some(false),
                _ => None,
            },
            page: page_number.get(),
            limit: page_size.get(),
        };
        set_is_loading.set(true);
        set_error.set(None);
        let s = service.clone();
        spawn_local(async move {
            let mut first_err: Option<String> = None;
            match s.fetch_options().await {
                Ok(v) => set_options.set(v),
                Err(e) => first_err = Some(e),
            }
            match s.fetch_programs(&query).await {
                Ok(v) => set_page_data.set(v),
                Err(e) => {
                    set_page_data.set(ProgramPageDto::default());
                    first_err = first_err.or(Some(e));
                }
            }
            if let Some(e) = first_err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    });

    let set_system = Callback::new(move |v: String| {
        set_system_filter.set(v);
        set_module_filter.set("ALL".to_string());
        set_page_number.set(1);
    });
    let set_module = Callback::new(move |v: String| {
        set_module_filter.set(v);
        set_page_number.set(1);
    });
    let set_search = Callback::new(move |v: String| {
        set_search_query.set(v);
        set_page_number.set(1);
    });
    let set_status = Callback::new(move |v: String| {
        set_status_filter.set(v);
        set_page_number.set(1);
    });
    let go_to_page = Callback::new(move |p: u64| set_page_number.set(p.max(1)));
    let set_limit = Callback::new(move |n: u64| {
        set_page_size.set(n.clamp(1, 200));
        set_page_number.set(1);
    });
    let refresh = Callback::new(move |()| set_refresh_trigger.update(|v| *v = v.wrapping_add(1)));

    let navigate = use_navigate();
    let open_create = Callback::new({
        let navigate = navigate.clone();
        move |()| {
            let mut parts = Vec::new();
            for (key, value) in [
                ("system_id", system_filter.get_untracked()),
                ("module_id", module_filter.get_untracked()),
            ] {
                if value != "ALL" && !value.is_empty() {
                    parts.push(format!("{key}={value}"));
                }
            }
            let path = supg002_program_path(None);
            if parts.is_empty() {
                navigate(&path, Default::default());
            } else {
                navigate(&format!("{path}?{}", parts.join("&")), Default::default());
            }
        }
    });
    let open_edit = Callback::new(move |p: ProgramDto| {
        navigate(&supg002_program_path(Some(&p.id)), Default::default())
    });

    let ctx = Supg002Context {
        options,
        page,
        system_filter,
        module_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_system,
        set_module,
        set_search,
        set_status,
        go_to_page,
        set_limit,
        refresh,
        open_create,
        open_edit,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Supg002Context
pub fn use_supg002() -> Supg002Context {
    use_context::<Supg002Context>()
        .expect("Supg002Context must be provided by provide_supg002_context()")
}
