//! DBPG004 form context: loads the lookup to edit plus the category list, saves with the loaded
//! `row_version`, and returns to the DBPG004 list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::db::dbpg004::models::{
    CreateLookupInput, LookupCategoryDto, LookupDto, UpdateLookupInput,
};
use crate::pages::db::dbpg004::services::Dbpg004Service;
use crate::routes::DBPG004;

/// State of the lookup form (`/db/dbpg004/lookup[/:id]`)
#[derive(Clone, Copy)]
pub struct Dbpg004FormContext {
    pub editing: ReadSignal<Option<LookupDto>>,
    pub categories: ReadSignal<Vec<LookupCategoryDto>>,
    /// Category to pre-select when creating (`?category=`)
    pub default_category: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub is_deleting: ReadSignal<bool>,
    pub save: Callback<CreateLookupInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// Provides the lookup form context; `id` is `Some` when editing.
pub fn provide_dbpg004_form_context(
    id: Option<String>,
    default_category: Option<String>,
) -> Dbpg004FormContext {
    let service = Dbpg004Service::new();
    let (editing, set_editing) = signal(Option::<LookupDto>::None);
    let (categories, set_categories) = signal(Vec::<LookupCategoryDto>::new());
    let (default_category, _) = signal(default_category);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(DBPG004, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            match s.fetch_categories().await {
                Ok(v) => set_categories.set(v),
                Err(e) => err = Some(e),
            }
            if let Some(id) = id {
                match s.fetch_lookup(&id).await {
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

    let delete_service = service.clone();
    let delete_id = id.clone();
    let delete = Callback::new(move |()| {
        let Some(id) = delete_id.clone() else { return };
        let s = delete_service.clone();
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_lookup(&id).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG004.delete", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let save = Callback::new(move |input: CreateLookupInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = match (id, current) {
                (Some(id), Some(lookup)) => {
                    s.update_lookup(
                        &id,
                        UpdateLookupInput {
                            active: input.active,
                            name_th: input.name_th,
                            name_en: input.name_en,
                            row_version: lookup.row_version,
                        },
                    )
                    .await
                }
                (Some(_), None) => Err(t_or("dbpg004.form.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s.create_lookup(input).await,
            };
            set_is_saving.set(false);
            match res {
                Ok(_) => {
                    track_action("DBPG004.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg004FormContext {
        editing,
        categories,
        default_category,
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

pub fn use_dbpg004_form() -> Dbpg004FormContext {
    use_context::<Dbpg004FormContext>()
        .expect("Dbpg004FormContext must be provided by provide_dbpg004_form_context()")
}
