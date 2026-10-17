//! SUPG004 list context: group / status filters, search and server-side paging of the accounts,


use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg004::models::{GroupOptionDto, UserAccountDto, UserPageDto};
use crate::pages::su::supg004::services::{Supg004Service, UserListQuery};
use crate::routes::supg004_user_path;

/// Rows per page of the account list
pub const PAGE_SIZE: u64 = 25;

/// Shared reactive state for the SUPG004 list page
#[derive(Clone, Copy)]
pub struct Supg004Context {
    pub groups: ReadSignal<Vec<GroupOptionDto>>,
    pub page: ReadSignal<UserPageDto>,
    pub group_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_group: Callback<String>,
    pub set_search: Callback<String>,
    pub set_status: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<UserAccountDto>,
}

/// Provides Supg004Context into the reactive tree
pub fn provide_supg004_context() -> Supg004Context {
    let service = Supg004Service::new();
    let (groups, set_groups) = signal(Vec::<GroupOptionDto>::new());
    let (page, set_page_data) = signal(UserPageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (group_filter, set_group_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    let list_service = service.clone();
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let query = UserListQuery {
            group_id: Some(group_filter.get()),
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
        let s = list_service.clone();
        spawn_local(async move {
            let mut first_err: Option<String> = None;
            match s.fetch_groups().await {
                Ok(v) => set_groups.set(v),
                Err(e) => first_err = Some(e),
            }
            match s.fetch_users(&query).await {
                Ok(v) => set_page_data.set(v),
                Err(e) => {
                    set_page_data.set(UserPageDto::default());
                    first_err = first_err.or(Some(e));
                }
            }
            if let Some(e) = first_err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    });

    let set_group = Callback::new(move |v: String| {
        set_group_filter.set(v);
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
        move |()| navigate(&supg004_user_path(None), Default::default())
    });
    let open_edit = Callback::new(move |u: UserAccountDto| {
        navigate(&supg004_user_path(Some(&u.id)), Default::default())
    });

    let ctx = Supg004Context {
        groups,
        page,
        group_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_group,
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

/// Hook to retrieve the current Supg004Context
pub fn use_supg004() -> Supg004Context {
    use_context::<Supg004Context>()
        .expect("Supg004Context must be provided by provide_supg004_context()")
}
