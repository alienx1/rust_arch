//! DBPG002 form contexts (branch / department): load the record + lookups, save with the loaded
//! `row_version`, and return to the DBPG002 list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::db::dbpg002::models::{
    BranchDto, BranchTypeOptionDto, CompanyOptionDto, CreateBranchInput, CreateDepartmentInput,
    DepartmentDto, UpdateBranchInput, UpdateDepartmentInput,
};
use crate::pages::db::dbpg002::services::Dbpg002Service;
use crate::routes::DBPG002;

/// State of the branch form (`/db/dbpg002/branch[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg002BranchFormContext {
    pub editing: ReadSignal<Option<BranchDto>>,
    pub companies: ReadSignal<Vec<CompanyOptionDto>>,
    pub branch_types: ReadSignal<Vec<BranchTypeOptionDto>>,
    /// Company to pre-select when creating (`?company_id=`)
    pub default_company_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<CreateBranchInput>,
    pub cancel: Callback<()>,
    pub delete: Callback<()>,
}

/// State of the department form (`/db/dbpg002/department[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg002DepartmentFormContext {
    pub editing: ReadSignal<Option<DepartmentDto>>,
    pub branches: ReadSignal<Vec<BranchDto>>,
    /// Branch to pre-select when creating (`?branch_id=`)
    pub default_branch_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<CreateDepartmentInput>,
    pub cancel: Callback<()>,
    pub delete: Callback<()>,
}

fn back_to_list() -> Callback<()> {
    let navigate = use_navigate();
    Callback::new(move |()| navigate(DBPG002, Default::default()))
}

/// Provides the branch form context; `id` is `Some` when editing.
pub fn provide_dbpg002_branch_form_context(
    id: Option<String>,
    default_company_id: Option<String>,
) -> Dbpg002BranchFormContext {
    let service = Dbpg002Service::new();
    let (editing, set_editing) = signal(Option::<BranchDto>::None);
    let (companies, set_companies) = signal(Vec::<CompanyOptionDto>::new());
    let (branch_types, set_branch_types) = signal(Vec::<BranchTypeOptionDto>::new());
    let (default_company_id, _) = signal(default_company_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let cancel = back_to_list();

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err = None;
            match s.fetch_company_options().await {
                Ok(v) => set_companies.set(v),
                Err(e) => err = Some(e),
            }
            match s.fetch_branch_types().await {
                Ok(v) => set_branch_types.set(v),
                Err(e) => err = err.or(Some(e)),
            }
            if let Some(id) = id {
                match s.fetch_branch(&id).await {
                    Ok(v) => set_editing.set(Some(v)),
                    Err(e) => err = err.or(Some(e)),
                }
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    let delete_id = id.clone();
    let delete_service = service.clone();
    let delete = Callback::new(move |()| {
        let s = delete_service.clone();
        let Some(id) = delete_id.clone() else { return };
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_branch(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG002.delete", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let save = Callback::new(move |input: CreateBranchInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(branch)) => {
                    s.update_branch(
                        &id,
                        UpdateBranchInput {
                            code: input.code,
                            master_lookup_id: input.master_lookup_id,
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: branch.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or(
                    "dbpg002.branch.error.not_loaded",
                    "ยังโหลดข้อมูลสาขาไม่สำเร็จ ไม่สามารถบันทึกได้",
                )),
                (None, _) => s.create_branch(input).await,
            };
            set_is_saving.set(false);
            match res {
                Ok(_) => {
                    track_action("DBPG002.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg002BranchFormContext {
        editing,
        companies,
        branch_types,
        default_company_id,
        is_loading,
        is_saving,
        is_deleting,
        error,
        save,
        cancel,
        delete,
    };
    provide_context(ctx);
    ctx
}

/// Provides the department form context; `id` is `Some` when editing. `company_id` narrows the
/// branch selector, `default_branch_id` pre-selects the parent branch.
pub fn provide_dbpg002_department_form_context(
    id: Option<String>,
    company_id: Option<String>,
    default_branch_id: Option<String>,
) -> Dbpg002DepartmentFormContext {
    let service = Dbpg002Service::new();
    let (editing, set_editing) = signal(Option::<DepartmentDto>::None);
    let (branches, set_branches) = signal(Vec::<BranchDto>::new());
    let (default_branch_id, _) = signal(default_branch_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let cancel = back_to_list();

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err = None;
            // Editing keeps every branch selectable (the department may move); creating narrows
            // to the company the list was filtered on.
            let company_filter = if id.is_some() { None } else { company_id };
            match s.fetch_branches(company_filter, None, Some(true)).await {
                Ok(v) => set_branches.set(v),
                Err(e) => err = Some(e),
            }
            if let Some(id) = id {
                match s.fetch_department(&id).await {
                    Ok(v) => set_editing.set(Some(v)),
                    Err(e) => err = err.or(Some(e)),
                }
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    let delete_id = id.clone();
    let delete_service = service.clone();
    let delete = Callback::new(move |()| {
        let s = delete_service.clone();
        let Some(id) = delete_id.clone() else { return };
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_department(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG002.delete", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let save = Callback::new(move |input: CreateDepartmentInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(dept)) => {
                    s.update_department(
                        &id,
                        UpdateDepartmentInput {
                            branch_id: input.branch_id,
                            code: input.code,
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: dept.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or(
                    "dbpg002.dept.error.not_loaded",
                    "ยังโหลดข้อมูลแผนกไม่สำเร็จ ไม่สามารถบันทึกได้",
                )),
                (None, _) => s.create_department(input).await,
            };
            set_is_saving.set(false);
            match res {
                Ok(_) => {
                    track_action("DBPG002.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg002DepartmentFormContext {
        editing,
        branches,
        default_branch_id,
        is_loading,
        is_saving,
        is_deleting,
        error,
        save,
        cancel,
        delete,
    };
    provide_context(ctx);
    ctx
}

pub fn use_dbpg002_branch_form() -> Dbpg002BranchFormContext {
    use_context::<Dbpg002BranchFormContext>()
        .expect("Dbpg002BranchFormContext must be provided by provide_dbpg002_branch_form_context()")
}

pub fn use_dbpg002_department_form() -> Dbpg002DepartmentFormContext {
    use_context::<Dbpg002DepartmentFormContext>().expect(
        "Dbpg002DepartmentFormContext must be provided by provide_dbpg002_department_form_context()",
    )
}
