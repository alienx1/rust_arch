//! DBPG004 list context: KPI summary, category filter, search, paging.
//!
//! Create/edit actions navigate to the DBPG004 form page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg004::models::{
    Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupPageDto,
};
use crate::pages::db::dbpg004::services::{Dbpg004Service, LookupListQuery};
use crate::routes::dbpg004_lookup_path;

/// Rows per page of the lookup list
pub const PAGE_SIZE: u64 = 25;

/// Shared reactive state for the DBPG004 list page
#[derive(Clone, Copy)]
pub struct Dbpg004Context {
    pub summary: ReadSignal<Dbpg004SummaryDto>,
    pub categories: ReadSignal<Vec<LookupCategoryDto>>,
    pub page: ReadSignal<LookupPageDto>,
    pub category_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Changing a filter returns to page 1
    pub set_category: Callback<String>,
    pub set_search: Callback<String>,
    pub set_status: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<LookupDto>,
}

/// Provides Dbpg004Context into the reactive tree
pub fn provide_dbpg004_context() -> Dbpg004Context {
    let service = Dbpg004Service::new();

    let (summary, set_summary) = signal(Dbpg004SummaryDto::default());
    let (categories, set_categories) = signal(Vec::<LookupCategoryDto>::new());
    let (page, set_page_data) = signal(LookupPageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (category_filter, set_category_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let query = LookupListQuery {
                category: Some(category_filter.get()),
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
                let mut keep = |e: String| {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                };
                match s.fetch_summary().await {
                    Ok(v) => set_summary.set(v),
                    Err(e) => keep(e),
                }
                match s.fetch_categories().await {
                    Ok(v) => set_categories.set(v),
                    Err(e) => keep(e),
                }
                match s.fetch_lookups(&query).await {
                    Ok(v) => set_page_data.set(v),
                    Err(e) => {
                        set_page_data.set(LookupPageDto::default());
                        keep(e);
                    }
                }
                if let Some(e) = first_err {
                    set_error.set(Some(e));
                }
                set_is_loading.set(false);
            });
        }
    });

    let set_category = Callback::new(move |v: String| {
        set_category_filter.set(v);
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
            // Pre-select the category currently chosen in the list filter
            let category = category_filter.get_untracked();
            let path = dbpg004_lookup_path(None);
            if category == "ALL" || category.is_empty() {
                navigate(&path, Default::default());
            } else {
                navigate(&format!("{path}?category={category}"), Default::default());
            }
        }
    });
    let open_edit = Callback::new(move |l: LookupDto| {
        navigate(&dbpg004_lookup_path(Some(&l.id)), Default::default())
    });

    let ctx = Dbpg004Context {
        summary,
        categories,
        page,
        category_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_category,
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

/// Hook to retrieve the current Dbpg004Context
pub fn use_dbpg004() -> Dbpg004Context {
    use_context::<Dbpg004Context>()
        .expect("Dbpg004Context must be provided by provide_dbpg004_context()")
}
