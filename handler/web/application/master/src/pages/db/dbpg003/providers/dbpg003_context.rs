//! DBPG003 list context: KPI summary, company/branch filters, search, paging.
//!
//! Create/edit actions navigate to the DBPG003 form pages.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, Dbpg003SummaryDto, EmployeeDto, EmployeePageDto,
};
use crate::pages::db::dbpg003::services::{Dbpg003Service, EmployeeListQuery};
use crate::routes::dbpg003_employee_path;

/// Rows per page of the employee list
pub const PAGE_SIZE: u64 = 25;

/// Shared reactive state for the DBPG003 list page
#[derive(Clone, Copy)]
pub struct Dbpg003Context {
    pub summary: ReadSignal<Dbpg003SummaryDto>,
    pub companies: ReadSignal<Vec<CompanyOptionDto>>,
    pub branches: ReadSignal<Vec<BranchOptionDto>>,
    pub page: ReadSignal<EmployeePageDto>,
    pub company_filter: ReadSignal<String>,
    pub branch_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Changing a filter returns to page 1; a company change also clears the branch filter
    pub set_company: Callback<String>,
    pub set_branch: Callback<String>,
    pub set_search: Callback<String>,
    pub set_status: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit: Callback<EmployeeDto>,
}

/// Provides Dbpg003Context into the reactive tree
pub fn provide_dbpg003_context() -> Dbpg003Context {
    let service = Dbpg003Service::new();

    let (summary, set_summary) = signal(Dbpg003SummaryDto::default());
    let (companies, set_companies) = signal(Vec::<CompanyOptionDto>::new());
    let (branches, set_branches) = signal(Vec::<BranchOptionDto>::new());
    let (page, set_page_data) = signal(EmployeePageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (company_filter, set_company_filter) = signal("ALL".to_string());
    let (branch_filter, set_branch_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let query = EmployeeListQuery {
                company_id: Some(company_filter.get()),
                branch_id: Some(branch_filter.get()),
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
                match s.fetch_company_options().await {
                    Ok(v) => set_companies.set(v),
                    Err(e) => keep(e),
                }
                match s.fetch_branch_options().await {
                    Ok(v) => set_branches.set(v),
                    Err(e) => keep(e),
                }
                match s.fetch_employees(&query).await {
                    Ok(v) => set_page_data.set(v),
                    Err(e) => {
                        set_page_data.set(EmployeePageDto::default());
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

    let set_company = Callback::new(move |v: String| {
        set_company_filter.set(v);
        set_branch_filter.set("ALL".to_string());
        set_page_number.set(1);
    });
    let set_branch = Callback::new(move |v: String| {
        set_branch_filter.set(v);
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
        move |()| navigate(&dbpg003_employee_path(None), Default::default())
    });
    let open_edit = Callback::new(move |e: EmployeeDto| {
        navigate(&dbpg003_employee_path(Some(&e.id)), Default::default())
    });

    let ctx = Dbpg003Context {
        summary,
        companies,
        branches,
        page,
        company_filter,
        branch_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_company,
        set_branch,
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

/// Hook to retrieve the current Dbpg003Context
pub fn use_dbpg003() -> Dbpg003Context {
    use_context::<Dbpg003Context>()
        .expect("Dbpg003Context must be provided by provide_dbpg003_context()")
}
