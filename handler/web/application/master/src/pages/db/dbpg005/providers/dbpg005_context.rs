//! DBPG005 list context: KPI counts, province/district cascade, search, paging per level.
//!
//! Create/edit actions navigate to the DBPG005 form page of the level being shown.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg005::models::{
    Dbpg005SummaryDto, GeoLevel, GeoNodeDto, GeoPageDto, PostalCodeDto, PostalPageDto,
};
use crate::pages::db::dbpg005::services::{Dbpg005Service, GeoListQuery};
use crate::routes::dbpg005_form_path;

/// Rows per page of the lists
pub const PAGE_SIZE: u64 = 25;
/// Page size that fits a whole selector level (77 provinces, <= ~50 districts per province)
const OPTIONS_LIMIT: u64 = 200;

/// Shared reactive state for the DBPG005 list page
#[derive(Clone, Copy)]
pub struct Dbpg005Context {
    pub summary: ReadSignal<Dbpg005SummaryDto>,
    /// All provinces (for the province filter)
    pub provinces: ReadSignal<Vec<GeoNodeDto>>,
    /// Districts of the selected province (for the district filter)
    pub districts: ReadSignal<Vec<GeoNodeDto>>,
    pub tab: ReadSignal<GeoLevel>,
    pub province_filter: ReadSignal<String>,
    pub district_filter: ReadSignal<String>,
    pub search_query: ReadSignal<String>,
    pub status_filter: ReadSignal<String>,
    /// Data of the province / district / sub-district tabs
    pub geo_page: ReadSignal<GeoPageDto>,
    /// Data of the postal-code tab
    pub postal_page: ReadSignal<PostalPageDto>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Changing a filter or the tab returns to page 1; a province change clears the district
    pub set_tab: Callback<GeoLevel>,
    pub set_province: Callback<String>,
    pub set_district: Callback<String>,
    pub set_search: Callback<String>,
    pub set_status: Callback<String>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_create: Callback<()>,
    pub open_edit_geo: Callback<GeoNodeDto>,
    pub open_edit_postal: Callback<PostalCodeDto>,
}

/// Provides Dbpg005Context into the reactive tree
pub fn provide_dbpg005_context() -> Dbpg005Context {
    let service = Dbpg005Service::new();

    let (summary, set_summary) = signal(Dbpg005SummaryDto::default());
    let (provinces, set_provinces) = signal(Vec::<GeoNodeDto>::new());
    let (districts, set_districts) = signal(Vec::<GeoNodeDto>::new());
    let (geo_page, set_geo_page) = signal(GeoPageDto::default());
    let (postal_page, set_postal_page) = signal(PostalPageDto::default());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (tab, set_tab_signal) = signal(GeoLevel::Province);
    let (province_filter, set_province_filter) = signal("ALL".to_string());
    let (district_filter, set_district_filter) = signal("ALL".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (status_filter, set_status_filter) = signal("ALL".to_string());
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    // KPI counts + province options
    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let s = service.clone();
            spawn_local(async move {
                match s.fetch_summary().await {
                    Ok(v) => set_summary.set(v),
                    Err(e) => set_error.set(Some(e)),
                }
                let q = GeoListQuery { page: 1, limit: OPTIONS_LIMIT, ..Default::default() };
                match s.fetch_provinces(&q).await {
                    Ok(v) => set_provinces.set(v.items),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    // District options follow the province filter
    Effect::new({
        let service = service.clone();
        move |_| {
            let province = province_filter.get();
            if province == "ALL" {
                set_districts.set(Vec::new());
                return;
            }
            let s = service.clone();
            spawn_local(async move {
                let q = GeoListQuery {
                    province_id: Some(province),
                    page: 1,
                    limit: OPTIONS_LIMIT,
                    ..Default::default()
                };
                match s.fetch_districts(&q).await {
                    Ok(v) => set_districts.set(v.items),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    // The list of the active tab
    Effect::new({
        let service = service.clone();
        move |_| {
            let _ = refresh_trigger.get();
            let level = tab.get();
            let query = GeoListQuery {
                province_id: Some(province_filter.get()),
                district_id: Some(district_filter.get()),
                search: Some(search_query.get()),
                active: match status_filter.get().as_str() {
                    "ACTIVE" => Some(true),
                    "INACTIVE" => Some(false),
                    _ => None,
                },
                page: page_number.get(),
                limit: page_size.get(),
                ..Default::default()
            };
            // A level only uses the filters above it
            let query = match level {
                GeoLevel::Province => GeoListQuery { province_id: None, district_id: None, ..query },
                GeoLevel::District => GeoListQuery { district_id: None, ..query },
                _ => query,
            };

            set_is_loading.set(true);
            set_error.set(None);
            let s = service.clone();
            spawn_local(async move {
                let result = match level {
                    GeoLevel::Province => s.fetch_provinces(&query).await.map(|p| set_geo_page.set(p)),
                    GeoLevel::District => s.fetch_districts(&query).await.map(|p| set_geo_page.set(p)),
                    GeoLevel::SubDistrict => {
                        s.fetch_sub_districts(&query).await.map(|p| set_geo_page.set(p))
                    }
                    GeoLevel::PostalCode => {
                        s.fetch_postal_codes(&query).await.map(|p| set_postal_page.set(p))
                    }
                };
                if let Err(e) = result {
                    set_geo_page.set(GeoPageDto::default());
                    set_postal_page.set(PostalPageDto::default());
                    set_error.set(Some(e));
                }
                set_is_loading.set(false);
            });
        }
    });

    let set_tab = Callback::new(move |level: GeoLevel| {
        set_geo_page.set(GeoPageDto::default());
        set_tab_signal.set(level);
        set_page_number.set(1);
    });
    let set_province = Callback::new(move |v: String| {
        set_province_filter.set(v);
        set_district_filter.set("ALL".to_string());
        set_page_number.set(1);
    });
    let set_district = Callback::new(move |v: String| {
        set_district_filter.set(v);
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
            // Carry the filters the user has narrowed down to into the new record
            let mut parts = Vec::new();
            for (key, value) in [
                ("province_id", province_filter.get_untracked()),
                ("district_id", district_filter.get_untracked()),
            ] {
                if value != "ALL" && !value.is_empty() {
                    parts.push(format!("{key}={value}"));
                }
            }
            let path = dbpg005_form_path(tab.get_untracked(), None);
            if parts.is_empty() {
                navigate(&path, Default::default());
            } else {
                navigate(&format!("{path}?{}", parts.join("&")), Default::default());
            }
        }
    });
    let open_edit_geo = Callback::new({
        let navigate = navigate.clone();
        move |n: GeoNodeDto| navigate(&dbpg005_form_path(tab.get_untracked(), Some(&n.id)), Default::default())
    });
    let open_edit_postal = Callback::new(move |p: PostalCodeDto| {
        navigate(&dbpg005_form_path(GeoLevel::PostalCode, Some(&p.id)), Default::default())
    });

    let ctx = Dbpg005Context {
        summary,
        provinces,
        districts,
        tab,
        province_filter,
        district_filter,
        search_query,
        status_filter,
        geo_page,
        postal_page,
        is_loading,
        error,
        set_tab,
        set_province,
        set_district,
        set_search,
        set_status,
        go_to_page,
        set_limit,
        refresh,
        open_create,
        open_edit_geo,
        open_edit_postal,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Dbpg005Context
pub fn use_dbpg005() -> Dbpg005Context {
    use_context::<Dbpg005Context>()
        .expect("Dbpg005Context must be provided by provide_dbpg005_context()")
}
