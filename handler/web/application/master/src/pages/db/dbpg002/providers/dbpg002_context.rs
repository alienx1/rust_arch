//! DBPG002 list context: KPI summary, company filter, branch / department lists.
//!
//! Create/edit actions navigate to the DBPG002 form pages.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::pages::db::dbpg002::models::{
    BranchDto, CompanyOptionDto, Dbpg002SummaryDto, DepartmentDto,
};
use crate::pages::db::dbpg002::services::Dbpg002Service;
use crate::routes::{dbpg002_branch_path, dbpg002_department_path};

/// Which list the page is showing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dbpg002Tab {
    Branches,
    Departments,
}

/// Shared reactive state for the DBPG002 list page
#[derive(Clone, Copy)]
pub struct Dbpg002Context {
    pub summary: ReadSignal<Dbpg002SummaryDto>,
    pub companies: ReadSignal<Vec<CompanyOptionDto>>,
    pub branches: ReadSignal<Vec<BranchDto>>,
    pub departments: ReadSignal<Vec<DepartmentDto>>,
    pub tab: ReadSignal<Dbpg002Tab>,
    pub company_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_tab: WriteSignal<Dbpg002Tab>,
    pub set_company_filter: WriteSignal<String>,
    pub set_search_query: WriteSignal<String>,
    pub set_status_filter: WriteSignal<String>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit_branch: Callback<BranchDto>,
    pub open_edit_department: Callback<DepartmentDto>,
    /// Add a department directly under the given branch id
    pub open_create_department_in: Callback<String>,
}

/// Provides Dbpg002Context into the reactive tree. `?company_id=` pre-selects the company filter.
pub fn provide_dbpg002_context() -> Dbpg002Context {
    let service = Dbpg002Service::new();

    let (summary, set_summary) = signal(Dbpg002SummaryDto::default());
    let (companies, set_companies) = signal(Vec::<CompanyOptionDto>::new());
    let (branches, set_branches) = signal(Vec::<BranchDto>::new());
    let (departments, set_departments) = signal(Vec::<DepartmentDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (tab, set_tab) = signal(Dbpg002Tab::Branches);

    let initial_company = use_query_map()
        .get_untracked()
        .get("company_id")
        .unwrap_or_else(|| "ALL".to_string());
    let (company_filter, set_company_filter) = signal(initial_company);
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let company = company_filter.get();
            let search = search_query.get();
            let active = match status_filter.get().as_str() {
                "ACTIVE" => Some(true),
                "INACTIVE" => Some(false),
                _ => None,
            };
            let company_param = (company != "ALL" && !company.is_empty()).then_some(company);
            let search_param = (!search.trim().is_empty()).then_some(search);

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
                match s.fetch_branches(company_param.clone(), search_param.clone(), active).await {
                    Ok(v) => set_branches.set(v),
                    Err(e) => {
                        set_branches.set(Vec::new());
                        keep(e);
                    }
                }
                match s.fetch_departments(company_param, search_param, active).await {
                    Ok(v) => set_departments.set(v),
                    Err(e) => {
                        set_departments.set(Vec::new());
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

    let refresh = Callback::new(move |()| set_refresh_trigger.update(|v| *v = v.wrapping_add(1)));

    let navigate = use_navigate();
    let open_create = Callback::new({
        let navigate = navigate.clone();
        move |()| {
            let company = company_filter.get_untracked();
            let has_company = company != "ALL" && !company.is_empty();
            let (path, key) = match tab.get_untracked() {
                Dbpg002Tab::Branches => (dbpg002_branch_path(None), "company_id"),
                Dbpg002Tab::Departments => (dbpg002_department_path(None), "company_id"),
            };
            if has_company {
                navigate(&format!("{path}?{key}={company}"), Default::default());
            } else {
                navigate(&path, Default::default());
            }
        }
    });
    let open_edit_branch = Callback::new({
        let navigate = navigate.clone();
        move |b: BranchDto| navigate(&dbpg002_branch_path(Some(&b.id)), Default::default())
    });
    let open_edit_department = Callback::new({
        let navigate = navigate.clone();
        move |d: DepartmentDto| navigate(&dbpg002_department_path(Some(&d.id)), Default::default())
    });
    let open_create_department_in = Callback::new(move |branch_id: String| {
        navigate(
            &format!("{}?branch_id={branch_id}", dbpg002_department_path(None)),
            Default::default(),
        )
    });

    let ctx = Dbpg002Context {
        summary,
        companies,
        branches,
        departments,
        tab,
        company_filter,
        search_query,
        status_filter,
        is_loading,
        error,
        set_tab,
        set_company_filter,
        set_search_query,
        set_status_filter,
        refresh,
        open_create,
        open_edit_branch,
        open_edit_department,
        open_create_department_in,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Dbpg002Context
pub fn use_dbpg002() -> Dbpg002Context {
    use_context::<Dbpg002Context>()
        .expect("Dbpg002Context must be provided by provide_dbpg002_context()")
}
