//! SUPG003 group form context: loads the group to edit, saves, and returns
//! to the list on success.

use web_package::core::i18n::t_or;
use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg003::models::{
    CreateGroupInput, GroupFormInput, PermissionGroupDto, UpdateGroupInput,
};
use crate::pages::su::supg003::services::Supg003Service;
use crate::routes::SUPG003;

/// State of the group form (`/su/supg003/group[/:id]`)
#[derive(Clone, Copy)]
pub struct Supg003FormContext {
    pub editing: ReadSignal<Option<PermissionGroupDto>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<GroupFormInput>,
    pub cancel: Callback<()>,
    pub delete: Callback<()>,
}

/// Provides the group form context; `id` is `Some` when editing.
pub fn provide_supg003_form_context(id: Option<String>) -> Supg003FormContext {
    let service = Supg003Service::new();
    let (editing, set_editing) = signal(Option::<PermissionGroupDto>::None);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(SUPG003, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            if let Some(id) = id {
                match s.fetch_group(&id).await {
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

    let delete = Callback::new({
        let service = service.clone();
        let id = id.clone();
        move |()| {
            let Some(id) = id.clone() else { return };
            let s = service.clone();
            set_is_deleting.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.delete_group(&id).await;
                set_is_deleting.set(false);
                match res {
                    Ok(()) => {
                        track_action("SUPG003.delete", "");
                        cancel.run(())
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    let save = Callback::new(move |input: GroupFormInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(group)) => s
                    .update_group(
                        &id,
                        UpdateGroupInput {
                            active: input.active,
                            row_version: group.row_version,
                        },
                    )
                    .await
                    .map(|_| ()),
                (Some(_), None) => Err(t_or("supg003.form.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s
                    .create_group(CreateGroupInput {
                        code: input.code,
                        active: input.active,
                    })
                    .await
                    .map(|_| ()),
            };
            set_is_saving.set(false);
            match res {
                Ok(()) => {
                    track_action("SUPG003.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Supg003FormContext { editing, is_loading, is_saving, is_deleting, error, save, cancel, delete };
    provide_context(ctx);
    ctx
}

pub fn use_supg003_form() -> Supg003FormContext {
    use_context::<Supg003FormContext>()
        .expect("Supg003FormContext must be provided by provide_supg003_form_context()")
}
