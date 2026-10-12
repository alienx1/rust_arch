//! i18n Core Module for WebAssembly / Leptos applications.

pub mod i18n_models;
pub mod i18n_service;
pub mod i18n_context;
pub mod language_switcher;

pub use i18n_models::{I18nDictionaryDto, LanguageInfo, ENGLISH_LANG_ID_STR, THAI_LANG_ID_STR};
pub use i18n_service::I18nService;
pub use i18n_context::{
    I18nContext, get_saved_language, localized, provide_i18n_context, save_language, t, t_fmt, t_or, use_i18n,
};
pub use language_switcher::LanguageSwitcher;
