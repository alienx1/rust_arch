//! SUPG001 form context: loads the unit to edit (and the systems for a module's selector), saves
//! with the loaded `row_version`, and returns to the list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use web_package::core::i18n::t_or;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg001::models::{
    CreateModuleInput, CreateSystemInput, ModuleDto, SystemDto, UnitFormInput, UnitKind,
    UpdateUnitInput,
};
use crate::pages::su::supg001::services::Supg001Service;
use crate::routes::SUPG001;

/// State of the system / module form (`/su/supg001/{system|module}[/:id]`)
#[derive(Clone, Copy)]
pub struct Supg001FormContext {
    pub kind: UnitKind,
    pub editing_system: ReadSignal<Option<SystemDto>>,
    pub editing_module: ReadSignal<Option<ModuleDto>>,
    pub systems: ReadSignal<Vec<SystemDto>>,
    /// System to pre-select when creating a module (`?system_id=`)
    pub default_system_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<UnitFormInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// Provides the form context for `kind`; `id` is `Some` when editing.
pub fn provide_supg001_form_context(
    kind: UnitKind,
    id: Option<String>,
    default_system_id: Option<String>,
) -> Supg001FormContext {
    let service = Supg001Service::new();
    let (editing_system, set_editing_system) = signal(Option::<SystemDto>::None);
    let (editing_module, set_editing_module) = signal(Option::<ModuleDto>::None);
    let (systems, set_systems) = signal(Vec::<SystemDto>::new());
    let (default_system_id, _) = signal(default_system_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(SUPG001, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            match s.fetch_systems().await {
                Ok(v) => set_systems.set(v),
                Err(e) => err = Some(e),
            }
            if let Some(id) = id {
                let loaded = match kind {
                    UnitKind::System => s.fetch_system(&id).await.map(|v| set_editing_system.set(Some(v))),
                    UnitKind::Module => s.fetch_module(&id).await.map(|v| set_editing_module.set(Some(v))),
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

    let delete = {
        let s = service.clone();
        let id = id.clone();
        Callback::new(move |()| {
            let Some(id) = id.clone() else { return };
            let s = s.clone();
            set_is_deleting.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = match kind {
                    UnitKind::System => s.delete_system(&id).await,
                    UnitKind::Module => s.delete_module(&id).await,
                };
                set_is_deleting.set(false);
                match res {
                    Ok(()) => {
                        track_action("SUPG001.delete", "");
                        cancel.run(())
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        })
    };

    let save = Callback::new(move |input: UnitFormInput| {
        let s = service.clone();
        let id = id.clone();
        let system = editing_system.get_untracked();
        let module = editing_module.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let update = |row_version: i64, input: &UnitFormInput| UpdateUnitInput {
                active: input.active,
                name_th: input.name_th.clone(),
                name_en: input.name_en.clone(),
                row_version,
            };
            let res: Result<(), String> = match (kind, id) {
                (UnitKind::System, None) => s
                    .create_system(CreateSystemInput {
                        code: input.code,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
                (UnitKind::Module, None) => s
                    .create_module(CreateModuleInput {
                        system_id: input.system_id,
                        code: input.code,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
                (UnitKind::System, Some(id)) => match system {
                    Some(v) => s.update_system(&id, update(v.row_version, &input)).await.map(|_| ()),
                    None => Err(t_or("supg001.form.error.not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                },
                (UnitKind::Module, Some(id)) => match module {
                    Some(v) => s.update_module(&id, update(v.row_version, &input)).await.map(|_| ()),
                    None => Err(t_or("supg001.form.error.not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                },
            };
            set_is_saving.set(false);
            match res {
                Ok(()) => {
                    track_action("SUPG001.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Supg001FormContext {
        kind,
        editing_system,
        editing_module,
        systems,
        default_system_id,
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

pub fn use_supg001_form() -> Supg001FormContext {
    use_context::<Supg001FormContext>()
        .expect("Supg001FormContext must be provided by provide_supg001_form_context()")
}
