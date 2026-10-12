//! Reactive Leptos Context provider and hooks for i18n dictionary consumption.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::RefCell;
use std::collections::HashMap;

use super::i18n_service::I18nService;

#[allow(dead_code)]
const STORAGE_LANG_KEY: &str = "arch_app_language";

thread_local! {
    /// Dictionary of the active language, mirrored out of the context so `t_or` / `t_fmt` also
    /// work where no reactive owner is around (async callbacks, services)
    static ACTIVE_DICTIONARY: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn lookup(key: &str) -> Option<String> {
    ACTIVE_DICTIONARY.with(|d| d.borrow().get(key).cloned())
}

#[derive(Clone, Copy)]
pub struct I18nContext {
    pub current_lang: ReadSignal<String>,
    pub set_lang: Callback<String>,
    pub dict: ReadSignal<HashMap<String, String>>,
    pub is_loading: ReadSignal<bool>,
}

impl I18nContext {
    /// Translates key `<category>.<key>`. If not found in dictionary, returns `key`.
    pub fn t(&self, key: &str) -> String {
        self.dict.with(|d| {
            d.get(key).cloned().unwrap_or_else(|| key.to_string())
        })
    }

    /// Translates key `<category>.<key>`. If not found, returns `fallback`.
    pub fn t_or(&self, key: &str, fallback: &str) -> String {
        self.dict.with(|d| {
            d.get(key).cloned().unwrap_or_else(|| fallback.to_string())
        })
    }
}

/// Retrieve saved language preference from localStorage or fallback to "th".
pub fn get_saved_language() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(val)) = storage.get_item(STORAGE_LANG_KEY) {
                    let v = val.trim().to_lowercase();
                    if v == "en" || v == "th" {
                        return v;
                    }
                }
            }
        }
    }
    "th".to_string()
}

/// Save language preference to localStorage.
pub fn save_language(code: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(STORAGE_LANG_KEY, code);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = code;
}

/// Provides i18n context at the root of the Leptos application.
pub fn provide_i18n_context() -> I18nContext {
    let service = I18nService::default();
    let initial_lang = get_saved_language();

    let (current_lang, set_current_lang) = signal(initial_lang);
    let (dict, set_dict) = signal(HashMap::<String, String>::new());
    let (is_loading, set_is_loading) = signal(true);

    // Effect triggers when current_lang changes, fetching from db_label
    let svc = service.clone();
    Effect::new(move |_| {
        let lang = current_lang.get();
        let svc = svc.clone();
        set_is_loading.set(true);

        spawn_local(async move {
            match svc.fetch_dictionary(&lang).await {
                Ok(dto) => {
                    ACTIVE_DICTIONARY.with(|d| *d.borrow_mut() = dto.labels.clone());
                    set_dict.set(dto.labels);
                    set_is_loading.set(false);
                }
                Err(err) => {
                    leptos::logging::warn!("i18n load failed for '{lang}': {err}");
                    set_is_loading.set(false);
                }
            }
        });
    });

    let set_lang = Callback::new(move |new_lang: String| {
        let code = if new_lang.trim().to_lowercase() == "en" { "en" } else { "th" };
        save_language(code);
        set_current_lang.set(code.to_string());
    });

    let ctx = I18nContext {
        current_lang,
        set_lang,
        dict,
        is_loading,
    };

    provide_context(ctx);
    ctx
}

/// Hook to consume `I18nContext`. Panics if context is not provided.
pub fn use_i18n() -> I18nContext {
    expect_context::<I18nContext>()
}

/// Translate key using active context, falling back to `key` if context or key missing.
pub fn t(key: &str) -> String {
    lookup(key).unwrap_or_else(|| key.to_string())
}

/// Translate key using active context, falling back to `fallback`.
///
/// Reads the active dictionary directly, so it is safe to call from anywhere; pages are
/// re-mounted when the language changes (see `App`), so a value computed once stays correct.
pub fn t_or(key: &str, fallback: &str) -> String {
    lookup(key).unwrap_or_else(|| fallback.to_string())
}

/// Like [`t_or`], then replaces each `{name}` placeholder with its value, e.g.
/// `t_fmt("common.error.delete_failed", "ลบ{what}ไม่สำเร็จ: {message}", &[("what", w), ("message", m)])`.
pub fn t_fmt(key: &str, fallback: &str, args: &[(&str, &str)]) -> String {
    let mut text = t_or(key, fallback);
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

/// The text of the active language out of a Thai/English pair, falling back to the other one
/// when it is empty. Data shown in tables, lists and dropdowns uses this so it appears in the
/// language of the system, one at a time.
pub fn localized(thai: &str, english: &str) -> String {
    let (first, second) = if get_saved_language() == "en" { (english, thai) } else { (thai, english) };
    if first.trim().is_empty() { second } else { first }.to_string()
}
