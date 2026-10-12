//! SUPG002 form context: loads the program to edit plus the selector options, saves with the
//! loaded `row_version`, and returns to the list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use web_package::core::i18n::t_or;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg002::models::{
    CreateProgramInput, ProgramDto, ProgramFormInput, ProgramOptionsDto, UpdateProgramInput,
};
use crate::pages::su::supg002::services::Supg002Service;
use crate::routes::SUPG002;

/// State of the program form (`/su/supg002/program[/:id]`)
#[derive(Clone, Copy)]
pub struct Supg002FormContext {
    pub editing: ReadSignal<Option<ProgramDto>>,
    pub options: ReadSignal<ProgramOptionsDto>,
    /// Pre-selected system / module when creating (`?system_id=`, `?module_id=`)
    pub default_system_id: ReadSignal<Option<String>>,
    pub default_module_id: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<ProgramFormInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

fn opt(v: &str) -> Option<String> {
    let v = v.trim();
    (!v.is_empty()).then(|| v.to_string())
}

/// Provides the form context; `id` is `Some` when editing.
pub fn provide_supg002_form_context(
    id: Option<String>,
    default_system_id: Option<String>,
    default_module_id: Option<String>,
) -> Supg002FormContext {
    let service = Supg002Service::new();
    let (editing, set_editing) = signal(Option::<ProgramDto>::None);
    let (options, set_options) = signal(ProgramOptionsDto::default());
    let (default_system_id, _) = signal(default_system_id);
    let (default_module_id, _) = signal(default_module_id);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(SUPG002, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            match s.fetch_options().await {
                Ok(v) => set_options.set(v),
                Err(e) => err = Some(e),
            }
            if let Some(id) = id {
                match s.fetch_program(&id).await {
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

    let delete = {
        let s = service.clone();
        let id = id.clone();
        Callback::new(move |()| {
            let Some(id) = id.clone() else { return };
            let s = s.clone();
            set_is_deleting.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.delete_program(&id).await;
                set_is_deleting.set(false);
                match res {
                    Ok(()) => {
                        track_action("SUPG002.delete", "");
                        cancel.run(())
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        })
    };

    let save = Callback::new(move |input: ProgramFormInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(program)) => s
                    .update_program(
                        &id,
                        UpdateProgramInput {
                            parent_id: opt(&input.parent_id),
                            path: opt(&input.path),
                            icon: opt(&input.icon),
                            sort_order: input.sort_order,
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: program.row_version,
                        },
                    )
                    .await
                    .map(|_| ()),
                (Some(_), None) => Err(t_or("supg002.form.error.not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s
                    .create_program(CreateProgramInput {
                        system_id: input.system_id,
                        module_id: input.module_id,
                        parent_id: opt(&input.parent_id),
                        code: input.code,
                        path: opt(&input.path),
                        icon: opt(&input.icon),
                        sort_order: input.sort_order,
                        active: input.active,
                        name_th: input.name_th,
                        name_en: input.name_en,
                    })
                    .await
                    .map(|_| ()),
            };
            set_is_saving.set(false);
            match res {
                Ok(()) => {
                    track_action("SUPG002.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Supg002FormContext {
        editing,
        options,
        default_system_id,
        default_module_id,
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

pub fn use_supg002_form() -> Supg002FormContext {
    use_context::<Supg002FormContext>()
        .expect("Supg002FormContext must be provided by provide_supg002_form_context()")
}
