//! DBPG005 form context: loads the record to edit plus the province/district/sub-district
//! selector options, saves with the loaded `row_version`, and returns to the list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::db::dbpg005::models::{
    CreateDistrictInput, CreatePostalCodeInput, CreateProvinceInput, CreateSubDistrictInput,
    GeoFormInput, GeoLevel, GeoNodeDto, PostalCodeDto, UpdateGeoNodeInput, UpdatePostalCodeInput,
};
use crate::pages::db::dbpg005::services::{Dbpg005Service, GeoListQuery};
use crate::routes::DBPG005;

/// Page size that fits a whole selector level
const OPTIONS_LIMIT: u64 = 200;

/// State of the geography form (`/db/dbpg005/{level}[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg005FormContext {
    pub level: GeoLevel,
    /// Province / district / sub-district being edited
    pub editing_node: ReadSignal<Option<GeoNodeDto>>,
    /// Postal code being edited
    pub editing_postal: ReadSignal<Option<PostalCodeDto>>,
    pub provinces: ReadSignal<Vec<GeoNodeDto>>,
    /// Districts of the province chosen in the form
    pub districts: ReadSignal<Vec<GeoNodeDto>>,
    /// Sub-districts of the district chosen in the form
    pub sub_districts: ReadSignal<Vec<GeoNodeDto>>,
    /// Pre-selected parents when creating (`?province_id=`, `?district_id=`)
    pub default_province_id: ReadSignal<Option<String>>,
    pub default_district_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub load_districts: Callback<String>,
    pub load_sub_districts: Callback<String>,
    pub save: Callback<GeoFormInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// Provides the form context for `level`; `id` is `Some` when editing.
pub fn provide_dbpg005_form_context(
    level: GeoLevel,
    id: Option<String>,
    default_province_id: Option<String>,
    default_district_id: Option<String>,
) -> Dbpg005FormContext {
    let service = Dbpg005Service::new();
    let (editing_node, set_editing_node) = signal(Option::<GeoNodeDto>::None);
    let (editing_postal, set_editing_postal) = signal(Option::<PostalCodeDto>::None);
    let (provinces, set_provinces) = signal(Vec::<GeoNodeDto>::new());
    let (districts, set_districts) = signal(Vec::<GeoNodeDto>::new());
    let (sub_districts, set_sub_districts) = signal(Vec::<GeoNodeDto>::new());
    let (default_province_id, _) = signal(default_province_id);
    let (default_district_id, _) = signal(default_district_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(DBPG005, Default::default()));

    let load_districts = Callback::new({
        let service = service.clone();
        move |province_id: String| {
            if province_id.is_empty() {
                set_districts.set(Vec::new());
                return;
            }
            let s = service.clone();
            spawn_local(async move {
                let q = GeoListQuery {
                    province_id: Some(province_id),
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
    let load_sub_districts = Callback::new({
        let service = service.clone();
        move |district_id: String| {
            if district_id.is_empty() {
                set_sub_districts.set(Vec::new());
                return;
            }
            let s = service.clone();
            spawn_local(async move {
                let q = GeoListQuery {
                    district_id: Some(district_id),
                    page: 1,
                    limit: OPTIONS_LIMIT,
                    ..Default::default()
                };
                match s.fetch_sub_districts(&q).await {
                    Ok(v) => set_sub_districts.set(v.items),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            let q = GeoListQuery { page: 1, limit: OPTIONS_LIMIT, ..Default::default() };
            match s.fetch_provinces(&q).await {
                Ok(v) => set_provinces.set(v.items),
                Err(e) => err = Some(e),
            }
            if let Some(id) = id {
                let loaded = match level {
                    GeoLevel::Province => s.fetch_province(&id).await.map(|n| set_editing_node.set(Some(n))),
                    GeoLevel::District => s.fetch_district(&id).await.map(|n| set_editing_node.set(Some(n))),
                    GeoLevel::SubDistrict => {
                        s.fetch_sub_district(&id).await.map(|n| set_editing_node.set(Some(n)))
                    }
                    GeoLevel::PostalCode => {
                        s.fetch_postal_code(&id).await.map(|p| set_editing_postal.set(Some(p)))
                    }
                };
                if let Err(e) = loaded {
                    err = err.or(Some(e));
                }
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    // Hard delete of the record being edited; the server answers 409 while it is still referenced
    let delete = Callback::new({
        let service = service.clone();
        let id = id.clone();
        move |()| {
            let Some(id) = id.clone() else { return };
            let s = service.clone();
            set_is_deleting.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = match level {
                    GeoLevel::Province => s.delete_province(&id).await,
                    GeoLevel::District => s.delete_district(&id).await,
                    GeoLevel::SubDistrict => s.delete_sub_district(&id).await,
                    GeoLevel::PostalCode => s.delete_postal_code(&id).await,
                };
                set_is_deleting.set(false);
                match res {
                    Ok(()) => {
                        track_action("DBPG005.delete", "");
                        cancel.run(())
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    let save = Callback::new(move |input: GeoFormInput| {
        let s = service.clone();
        let id = id.clone();
        let node = editing_node.get_untracked();
        let postal = editing_postal.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let opt = |v: &str| (!v.trim().is_empty()).then(|| v.trim().to_string());
            let res: Result<(), String> = match (level, id) {
                (GeoLevel::Province, None) => s
                    .create_province(CreateProvinceInput {
                        code: input.code,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
                (GeoLevel::District, None) => s
                    .create_district(CreateDistrictInput {
                        province_id: input.province_id,
                        code: input.code,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
                (GeoLevel::SubDistrict, None) => s
                    .create_sub_district(CreateSubDistrictInput {
                        district_id: input.district_id,
                        code: input.code,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
                (GeoLevel::PostalCode, None) => s
                    .create_postal_code(CreatePostalCodeInput {
                        province_id: input.province_id,
                        district_id: opt(&input.district_id),
                        sub_district_id: opt(&input.sub_district_id),
                        postal_code: input.postal_code,
                        active: input.active,
                    })
                    .await
                    .map(|_| ()),
                (GeoLevel::PostalCode, Some(id)) => match postal {
                    Some(p) => s
                        .update_postal_code(
                            &id,
                            UpdatePostalCodeInput {
                                postal_code: input.postal_code,
                                active: input.active,
                                row_version: p.row_version,
                            },
                        )
                        .await
                        .map(|_| ()),
                    None => Err(t_or("dbpg005.form.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                },
                (level, Some(id)) => match node {
                    Some(n) => {
                        let update = UpdateGeoNodeInput {
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: n.row_version,
                        };
                        match level {
                            GeoLevel::Province => s.update_province(&id, update).await.map(|_| ()),
                            GeoLevel::District => s.update_district(&id, update).await.map(|_| ()),
                            _ => s.update_sub_district(&id, update).await.map(|_| ()),
                        }
                    }
                    None => Err(t_or("dbpg005.form.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                },
            };
            set_is_saving.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG005.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg005FormContext {
        level,
        editing_node,
        editing_postal,
        provinces,
        districts,
        sub_districts,
        default_province_id,
        default_district_id,
        is_loading,
        is_saving,
        is_deleting,
        error,
        load_districts,
        load_sub_districts,
        save,
        delete,
        cancel,
    };
    provide_context(ctx);
    ctx
}

pub fn use_dbpg005_form() -> Dbpg005FormContext {
    use_context::<Dbpg005FormContext>()
        .expect("Dbpg005FormContext must be provided by provide_dbpg005_form_context()")
}
