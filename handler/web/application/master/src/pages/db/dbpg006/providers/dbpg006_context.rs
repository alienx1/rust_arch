//! DBPG006 list context: category filter, search, server paging.
//!
//! Create/edit actions navigate to the DBPG006 form page.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg006::models::{LabelCategoryDto, LabelEntryDto, LabelPageDto};
use crate::pages::db::dbpg006::services::{Dbpg006Service, LabelListQuery};
use crate::routes::dbpg006_label_path;

/// Rows per page of the label list
pub const PAGE_SIZE: u64 = 25;

/// Shared reactive state for the DBPG006 list page
#[derive(Clone, Copy)]
pub struct Dbpg006Context {
    pub categories: ReadSignal<Vec<LabelCategoryDto>>,
    pub page: ReadSignal<LabelPageDto>,
    pub category_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Changing a filter returns to page 1
    pub set_category: Callback<String>,
    pub set_search: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<LabelEntryDto>,
}

/// Provides Dbpg006Context into the reactive tree
pub fn provide_dbpg006_context() -> Dbpg006Context {
    let service = Dbpg006Service::new();

    let (categories, set_categories) = signal(Vec::<LabelCategoryDto>::new());
    let (page, set_page_data) = signal(LabelPageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (category_filter, set_category_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let query = LabelListQuery {
                category: Some(category_filter.get()),
                search: Some(search_query.get()),
                page: page_number.get(),
                limit: page_size.get(),
            };

            set_is_loading.set(true);
            set_error.set(None);
            let s = service.clone();
            spawn_local(async move {
                let mut first_err: Option<String> = None;
                match s.fetch_categories().await {
                    Ok(v) => set_categories.set(v),
                    Err(e) => first_err = Some(e),
                }
                match s.fetch_labels(&query).await {
                    Ok(v) => set_page_data.set(v),
                    Err(e) => {
                        set_page_data.set(LabelPageDto::default());
                        first_err = first_err.or(Some(e));
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
            let preset = (category != "ALL" && !category.is_empty()).then_some(category);
            navigate(&dbpg006_label_path(preset.as_deref(), None), Default::default());
        }
    });
    let open_edit = Callback::new(move |e: LabelEntryDto| {
        navigate(&dbpg006_label_path(Some(&e.category), Some(&e.key)), Default::default())
    });

    let ctx = Dbpg006Context {
        categories,
        page,
        category_filter,
        search_query,
        is_loading,
        error,
        set_category,
        set_search,
        go_to_page,
        set_limit,
        refresh,
        open_create,
        open_edit,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Dbpg006Context
pub fn use_dbpg006() -> Dbpg006Context {
    use_context::<Dbpg006Context>()
        .expect("Dbpg006Context must be provided by provide_dbpg006_context()")
}
