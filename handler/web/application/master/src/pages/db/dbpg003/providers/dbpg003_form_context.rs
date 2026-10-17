//! DBPG003 form context: loads the employee to edit plus every lookup, saves with the loaded
//! `row_version`, and returns to the DBPG003 list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;
use web_package::ui::component::{ContactItem, read_image_file};

use crate::pages::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeInput, DepartmentOptionDto, EmployeeDto,
    PrefixOptionDto, UpdateEmployeeInput,
};
use crate::contacts::{load_contacts, save_contacts};
use crate::pages::db::dbpg003::services::{DBPG003_API, Dbpg003Service};
use crate::routes::DBPG003;

/// State of the employee form (`/db/dbpg003/employee[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg003FormContext {
    pub editing: ReadSignal<Option<EmployeeDto>>,
    /// Contacts of the employee (phone, e-mail, ID card ...), edited in the form and saved with it
    pub contacts: RwSignal<Vec<ContactItem>>,
    pub companies: ReadSignal<Vec<CompanyOptionDto>>,
    pub branches: ReadSignal<Vec<BranchOptionDto>>,
    pub departments: ReadSignal<Vec<DepartmentOptionDto>>,
    pub prefixes: ReadSignal<Vec<PrefixOptionDto>>,
    /// Company to pre-select when creating (`?company_id=`)
    pub default_company_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub is_deleting: ReadSignal<bool>,
    /// A photo upload / removal is running
    pub is_image_busy: ReadSignal<bool>,
    /// Uploads the chosen file as the employee's photo (editing only)
    pub upload_image: Callback<web_sys::File>,
    pub remove_image: Callback<()>,
    pub save: Callback<CreateEmployeeInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// Provides the employee form context; `id` is `Some` when editing.
pub fn provide_dbpg003_form_context(
    id: Option<String>,
    default_company_id: Option<String>,
) -> Dbpg003FormContext {
    let service = Dbpg003Service::new();
    let (editing, set_editing) = signal(Option::<EmployeeDto>::None);
    let (companies, set_companies) = signal(Vec::<CompanyOptionDto>::new());
    let (branches, set_branches) = signal(Vec::<BranchOptionDto>::new());
    let (departments, set_departments) = signal(Vec::<DepartmentOptionDto>::new());
    let (prefixes, set_prefixes) = signal(Vec::<PrefixOptionDto>::new());
    let contacts = RwSignal::new(Vec::<ContactItem>::new());
    if let Some(id) = id.clone() {
        load_contacts(DBPG003_API, format!("/employees/{id}"), contacts);
    }
    let (default_company_id, _) = signal(default_company_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(DBPG003, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            let mut keep = |e: String| {
                if err.is_none() {
                    err = Some(e);
                }
            };
            match s.fetch_company_options().await {
                Ok(v) => set_companies.set(v),
                Err(e) => keep(e),
            }
            match s.fetch_branch_options().await {
                Ok(v) => set_branches.set(v),
                Err(e) => keep(e),
            }
            match s.fetch_department_options().await {
                Ok(v) => set_departments.set(v),
                Err(e) => keep(e),
            }
            match s.fetch_prefixes().await {
                Ok(v) => set_prefixes.set(v),
                Err(e) => keep(e),
            }
            if let Some(id) = id {
                match s.fetch_employee(&id).await {
                    Ok(v) => set_editing.set(Some(v)),
                    Err(e) => keep(e),
                }
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    let image_service = service.clone();
    let image_id = id.clone();
    let delete_service = service.clone();
    let delete_id = id.clone();
    let delete = Callback::new(move |()| {
        let Some(id) = delete_id.clone() else { return };
        let s = delete_service.clone();
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_employee(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG003.delete", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let save = Callback::new(move |input: CreateEmployeeInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(employee)) => {
                    s.update_employee(
                        &id,
                        UpdateEmployeeInput {
                            department_id: input.department_id,
                            prefix_id: input.prefix_id,
                            code: input.code,
                            active: input.active,
                            first_name_th: input.first_name_th,
                            middle_name_th: input.middle_name_th,
                            last_name_th: input.last_name_th,
                            nickname_th: input.nickname_th,
                            first_name_en: input.first_name_en,
                            middle_name_en: input.middle_name_en,
                            last_name_en: input.last_name_en,
                            nickname_en: input.nickname_en,
                            row_version: employee.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or(
                    "dbpg003.error.not_loaded",
                    "ยังโหลดข้อมูลพนักงานไม่สำเร็จ ไม่สามารถบันทึกได้",
                )),
                (None, _) => s.create_employee(input).await,
            };
            let res = match res {
                Ok(saved) => save_contacts(DBPG003_API, &format!("/employees/{}", saved.id), &contacts.get_untracked())
                    .await
                    .map_err(|e| format!("{}: {e}", t_or("common.contact.saved_partly", "บันทึกข้อมูลแล้ว แต่บันทึกช่องทางติดต่อไม่สำเร็จ"))),
                Err(e) => Err(e),
            };
            set_is_saving.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG003.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let (is_image_busy, set_is_image_busy) = signal(false);
    let upload_image = {
        let (s, id) = (image_service.clone(), image_id.clone());
        Callback::new(move |file: web_sys::File| {
            let (s, Some(id)) = (s.clone(), id.clone()) else { return };
            set_is_image_busy.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = match read_image_file(file).await {
                    Ok(picked) => s.set_employee_image(&id, Some(picked)).await,
                    Err(e) => Err(e),
                };
                set_is_image_busy.set(false);
                match res {
                    Ok(employee) => set_editing.set(Some(employee)),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        })
    };
    let remove_image = {
        let (s, id) = (image_service.clone(), image_id.clone());
        Callback::new(move |()| {
            let (s, Some(id)) = (s.clone(), id.clone()) else { return };
            set_is_image_busy.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.set_employee_image(&id, None).await;
                set_is_image_busy.set(false);
                match res {
                    Ok(employee) => set_editing.set(Some(employee)),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        })
    };

    let ctx = Dbpg003FormContext {
        contacts,
        is_image_busy,
        upload_image,
        remove_image,
        editing,
        companies,
        branches,
        departments,
        prefixes,
        default_company_id,
        is_loading,
        is_saving,
        error,
        is_deleting,
        save,
        delete,
        cancel,
    };
    provide_context(ctx);
    ctx
}

pub fn use_dbpg003_form() -> Dbpg003FormContext {
    use_context::<Dbpg003FormContext>()
        .expect("Dbpg003FormContext must be provided by provide_dbpg003_form_context()")
}
