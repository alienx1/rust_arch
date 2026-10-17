//! DBPG006 form context: loads the languages, the categories and (when editing) the entry, saves
//! with create or update, and returns to the DBPG006 list on success.

use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::db::dbpg006::models::{
    LabelCategoryDto, LabelEntryDto, LanguageOptionDto, SaveLabelInput,
};
use crate::pages::db::dbpg006::services::Dbpg006Service;
use crate::routes::DBPG006;

/// State of the label form (`/db/dbpg006/label[?category=&key=]`)
#[derive(Clone, Copy)]
pub struct Dbpg006FormContext {
    /// `Some((category, key))` when editing an entry
    pub target: ReadSignal<Option<(String, String)>>,
    pub editing: ReadSignal<Option<LabelEntryDto>>,
    pub languages: ReadSignal<Vec<LanguageOptionDto>>,
    pub categories: ReadSignal<Vec<LabelCategoryDto>>,
    /// Category to pre-select when creating (`?category=`)
    pub default_category: ReadSignal<Option<String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub save: Callback<SaveLabelInput>,
    pub delete: Callback<()>,
    pub cancel: Callback<()>,
}

/// Provides the label form context; `target` is `Some((category, key))` when editing.
pub fn provide_dbpg006_form_context(
    target: Option<(String, String)>,
    default_category: Option<String>,
) -> Dbpg006FormContext {
    let service = Dbpg006Service::new();
    let (editing, set_editing) = signal(Option::<LabelEntryDto>::None);
    let (languages, set_languages) = signal(Vec::<LanguageOptionDto>::new());
    let (categories, set_categories) = signal(Vec::<LabelCategoryDto>::new());
    let (default_category, _) = signal(default_category);
    let (target_signal, _) = signal(target.clone());
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(DBPG006, Default::default()));

    {
        let s = service.clone();
        let target = target.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            match s.fetch_languages().await {
                Ok(v) => set_languages.set(v),
                Err(e) => err = Some(e),
            }
            match s.fetch_categories().await {
                Ok(v) => set_categories.set(v),
                Err(e) => err = err.or(Some(e)),
            }
            if let Some((category, key)) = target {
                match s.fetch_label(&category, &key).await {
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
    let delete_target = target.clone();
    let delete = Callback::new(move |()| {
        let Some((category, key)) = delete_target.clone() else { return };
        let s = delete_service.clone();
        set_is_deleting.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = s.delete_label(&category, &key).await;
            set_is_deleting.set(false);
            match res {
                Ok(()) => {
                    track_action("DBPG006.delete", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let is_edit = target.is_some();
    let save = Callback::new(move |mut input: SaveLabelInput| {
        let s = service.clone();
        // The version the entry was loaded with travels with the save
        input.row_version = editing.get_untracked().map(|e| e.row_version).unwrap_or(0);
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res = if is_edit { s.update_label(input).await } else { s.create_label(input).await };
            set_is_saving.set(false);
            match res {
                Ok(_) => {
                    track_action("DBPG006.save", "");
                    cancel.run(())
                }
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Dbpg006FormContext {
        target: target_signal,
        editing,
        languages,
        categories,
        default_category,
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

pub fn use_dbpg006_form() -> Dbpg006FormContext {
    use_context::<Dbpg006FormContext>()
        .expect("Dbpg006FormContext must be provided by provide_dbpg006_form_context()")
}
