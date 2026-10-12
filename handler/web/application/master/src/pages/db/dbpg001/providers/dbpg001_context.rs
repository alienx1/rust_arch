//! DBPG001 Reactive Contexts & Providers
//!
//! One context per detail form (organization / company): loads the record to edit plus the
//! lookups, saves it with the loaded `row_version`, and returns to DBPG000 on success.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use web_package::core::i18n::t_or;
use crate::pages::db::dbpg001::models::{
    CompanyDto, CompanyTypeOptionDto, CreateCompanyInput, CreateOrgInput, OrganizationDto,
    UpdateCompanyInput, UpdateOrgInput,
};
use crate::pages::db::dbpg001::services::Dbpg001Service;
use crate::routes::DBPG000;

/// State of the organization form (`/db/dbpg001/org[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg001OrgContext {
    /// Record being edited (`None` while creating or before it has loaded)
    pub editing: ReadSignal<Option<OrganizationDto>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub is_deleting: ReadSignal<bool>,
    pub save: Callback<CreateOrgInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// State of the company form (`/db/dbpg001/company[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg001CompanyContext {
    pub editing: ReadSignal<Option<CompanyDto>>,
    pub organizations: ReadSignal<Vec<OrganizationDto>>,
    pub company_types: ReadSignal<Vec<CompanyTypeOptionDto>>,
    /// Parent organization to pre-select when creating (`?organization_id=`)
    pub default_org_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub is_deleting: ReadSignal<bool>,
    pub save: Callback<CreateCompanyInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

fn back_to_list_callback() -> Callback<()> {
    let navigate = use_navigate();
    Callback::new(move |()| navigate(DBPG000, Default::default()))
}

/// Provides the organization form context; `id` is `Some` when editing.
pub fn provide_dbpg001_org_context(id: Option<String>) -> Dbpg001OrgContext {
    let service = Dbpg001Service::new();
    let (editing, set_editing) = signal(Option::<OrganizationDto>::None);
    let (is_loading, set_is_loading) = signal(id.is_some());
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let cancel = back_to_list_callback();
    let delete_id = id.clone();

    if let Some(id) = id.clone() {
        let s = service.clone();
        spawn_local(async move {
            match s.fetch_organization(&id).await {
                Ok(org) => set_editing.set(Some(org)),
                Err(e) => set_error.set(Some(e)),
            }
            set_is_loading.set(false);
        });
    }

    let save_service = service.clone();
    let save = Callback::new(move |input: CreateOrgInput| {
        let s = save_service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(org)) => {
                    s.update_organization(
                        &id,
                        UpdateOrgInput {
                            code: input.code,
                            registration_number: input.registration_number,
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: org.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or("dbpg001.org.error.not_loaded", "ยังโหลดข้อมูลองค์กรไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s.create_organization(input).await,
            };
            set_is_saving.set(false);
            match res {
                Ok(_) => cancel.run(()),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });


    let delete = Callback::new(move |()| {
        let s = service.clone();
        let Some(id) = delete_id.clone() else { return };
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_organization(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => cancel.run(()),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg001OrgContext {
        editing,
        is_loading,
        is_saving,
        is_deleting,
        error,
        save,
        delete,
        cancel,
    };
    provide_context(ctx);
    ctx
}

/// Provides the company form context; `id` is `Some` when editing.
pub fn provide_dbpg001_company_context(
    id: Option<String>,
    default_org_id: Option<String>,
) -> Dbpg001CompanyContext {
    let service = Dbpg001Service::new();
    let (editing, set_editing) = signal(Option::<CompanyDto>::None);
    let (organizations, set_organizations) = signal(Vec::<OrganizationDto>::new());
    let (company_types, set_company_types) = signal(Vec::<CompanyTypeOptionDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (default_org_id, _) = signal(default_org_id);
    let cancel = back_to_list_callback();
    let delete_id = id.clone();

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err = None;
            match s.fetch_organizations().await {
                Ok(orgs) => set_organizations.set(orgs),
                Err(e) => err = Some(e),
            }
            match s.fetch_company_types().await {
                Ok(types) => set_company_types.set(types),
                Err(e) => err = err.or(Some(e)),
            }
            if let Some(id) = id {
                match s.fetch_company(&id).await {
                    Ok(company) => set_editing.set(Some(company)),
                    Err(e) => err = err.or(Some(e)),
                }
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    let save_service = service.clone();
    let save = Callback::new(move |input: CreateCompanyInput| {
        let s = save_service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(company)) => {
                    s.update_company(
                        &id,
                        UpdateCompanyInput {
                            organization_id: input.organization_id,
                            code: input.code,
                            master_lookup_id: input.master_lookup_id,
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: company.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or("dbpg001.company.error.not_loaded", "ยังโหลดข้อมูลบริษัทไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s.create_company(input).await,
            };
            set_is_saving.set(false);
            match res {
                Ok(_) => cancel.run(()),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });


    let delete = Callback::new(move |()| {
        let s = service.clone();
        let Some(id) = delete_id.clone() else { return };
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_company(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => cancel.run(()),
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg001CompanyContext {
        editing,
        organizations,
        company_types,
        default_org_id,
        is_loading,
        is_saving,
        is_deleting,
        error,
        save,
        delete,
        cancel,
    };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the organization form context
pub fn use_dbpg001_org() -> Dbpg001OrgContext {
    use_context::<Dbpg001OrgContext>()
        .expect("Dbpg001OrgContext must be provided by provide_dbpg001_org_context()")
}

/// Hook to retrieve the company form context
pub fn use_dbpg001_company() -> Dbpg001CompanyContext {
    use_context::<Dbpg001CompanyContext>()
        .expect("Dbpg001CompanyContext must be provided by provide_dbpg001_company_context()")
}
