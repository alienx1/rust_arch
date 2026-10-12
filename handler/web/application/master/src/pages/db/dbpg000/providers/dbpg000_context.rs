//! DBPG000 Reactive Context & Provider
//!
//! Provides synchronized state for the organization selector, company list and search filters.
//! Create/edit actions navigate to the DBPG001 detail pages.

use leptos::prelude::*;
use leptos::task::spawn_local;

use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg000::models::{
    CompanyDto, CompanyTypeOptionDto, OrganizationDto, OrganizationSummaryDto,
};
use crate::routes::{dbpg001_company_path, dbpg001_org_path};
use crate::pages::db::dbpg000::services::Dbpg000Service;

/// Shared reactive state context for the organization feature
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Dbpg000Context {
    pub organizations: ReadSignal<Vec<OrganizationDto>>,
    pub companies: ReadSignal<Vec<CompanyDto>>,
    pub company_types: ReadSignal<Vec<CompanyTypeOptionDto>>,
    pub summary: ReadSignal<OrganizationSummaryDto>,
    pub selected_org_id: Signal<String>,
    pub search_query: Signal<String>,
    pub status_filter: Signal<String>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub set_selected_org_id: WriteSignal<String>,
    pub set_search_query: WriteSignal<String>,
    pub set_status_filter: WriteSignal<String>,
    pub refresh: Callback<()>,
    pub open_create_org: Callback<()>,
    pub open_edit_org: Callback<OrganizationDto>,
    pub open_create_company: Callback<()>,
    pub open_edit_company: Callback<CompanyDto>,
}

/// Provides Dbpg000Context into the reactive tree
pub fn provide_dbpg000_context() -> Dbpg000Context {
    let service = Dbpg000Service::new();

    let (organizations, set_organizations) = signal(Vec::<OrganizationDto>::new());
    let (companies, set_companies) = signal(Vec::<CompanyDto>::new());
    let (company_types, set_company_types) = signal(Vec::<CompanyTypeOptionDto>::new());
    let (summary, set_summary) = signal(OrganizationSummaryDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (selected_org_id_raw, set_selected_org_id) = signal("ALL".to_string());
    let selected_org_id = Signal::derive(move || selected_org_id_raw.get());

    let (search_query_raw, set_search_query) = signal("".to_string());
    let search_query = Signal::derive(move || search_query_raw.get());

    let (status_filter_raw, set_status_filter) = signal("ALL".to_string());
    let status_filter = Signal::derive(move || status_filter_raw.get());

    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    // Initial load and reload on trigger, selected_org, search_query, status_filter
    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let current_org = selected_org_id_raw.get();
            let current_search = search_query_raw.get();
            let current_status = status_filter_raw.get();

            let active_filter = match current_status.as_str() {
                "ACTIVE" => Some(true),
                "INACTIVE" => Some(false),
                _ => None,
            };

            set_is_loading.set(true);
            set_error.set(None);
            let s = service.clone();

            spawn_local(async move {
                let orgs_res = s.fetch_organizations().await;
                let types_res = s.fetch_company_types().await;
                let sum_res = s.fetch_summary().await;

                let org_param = if current_org == "ALL" {
                    None
                } else {
                    Some(current_org)
                };

                let search_param = if current_search.trim().is_empty() {
                    None
                } else {
                    Some(current_search)
                };

                let comps_res = s.fetch_companies(org_param, search_param, active_filter).await;

                let mut fetch_err = None;

                match orgs_res {
                    Ok(orgs) => set_organizations.set(orgs),
                    Err(e) => {
                        set_organizations.set(Vec::new());
                        fetch_err = Some(e);
                    }
                }
                match types_res {
                    Ok(types) => set_company_types.set(types),
                    Err(e) => {
                        if fetch_err.is_none() {
                            fetch_err = Some(e);
                        }
                    }
                }
                match sum_res {
                    Ok(sum) => set_summary.set(sum),
                    Err(e) => {
                        if fetch_err.is_none() {
                            fetch_err = Some(e);
                        }
                    }
                }
                match comps_res {
                    Ok(comps) => set_companies.set(comps),
                    Err(e) => {
                        set_companies.set(Vec::new());
                        if fetch_err.is_none() {
                            fetch_err = Some(e);
                        }
                    }
                }

                if let Some(err) = fetch_err {
                    set_error.set(Some(err));
                }

                set_is_loading.set(false);
            });
        }
    });

    let refresh = Callback::new(move |()| {
        set_refresh_trigger.update(|v| *v = v.wrapping_add(1));
    });

    let navigate = use_navigate();
    let open_create_org = Callback::new({
        let navigate = navigate.clone();
        move |()| navigate(&dbpg001_org_path(None), Default::default())
    });
    let open_edit_org = Callback::new({
        let navigate = navigate.clone();
        move |org: OrganizationDto| navigate(&dbpg001_org_path(Some(&org.id)), Default::default())
    });
    let open_create_company = Callback::new({
        let navigate = navigate.clone();
        move |()| {
            // Pre-select the parent organization currently chosen in the list filter.
            let org = selected_org_id_raw.get_untracked();
            let path = dbpg001_company_path(None);
            if org == "ALL" || org.is_empty() {
                navigate(&path, Default::default());
            } else {
                navigate(&format!("{path}?organization_id={org}"), Default::default());
            }
        }
    });
    let open_edit_company = Callback::new(move |company: CompanyDto| {
        navigate(&dbpg001_company_path(Some(&company.id)), Default::default())
    });

    let ctx = Dbpg000Context {
        organizations,
        companies,
        company_types,
        summary,
        selected_org_id,
        search_query,
        status_filter,
        is_loading,
        error,
        set_selected_org_id,
        set_search_query,
        set_status_filter,
        refresh,
        open_create_org,
        open_edit_org,
        open_create_company,
        open_edit_company,
    };

    provide_context(ctx);
    ctx
}

/// Convenience hook to retrieve the current Dbpg000Context
pub fn use_dbpg000() -> Dbpg000Context {
    use_context::<Dbpg000Context>()
        .expect("Dbpg000Context must be provided by provide_dbpg000_context()")
}
