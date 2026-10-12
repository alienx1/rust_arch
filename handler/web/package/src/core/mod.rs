//! Core infrastructure utilities for Leptos WebAssembly: HTTP requestor, Ory Kratos auth, and errors.

pub mod auth;
pub mod errors;
pub mod i18n;
pub mod requestor;

pub use auth::{
    AuthContext, KratosClient, KratosIdentity, KratosLoginFlow, KratosLogoutFlow,
    KratosRegistrationFlow, KratosSession, KratosUiContainer, KratosUiMessage, KratosUiNode,
    KratosUiNodeAttributes, LoginPayload, provide_auth_context, use_auth,
};
pub use errors::{AuthError, HttpError};
pub use i18n::{
    I18nContext, I18nDictionaryDto, I18nService, LanguageInfo, LanguageSwitcher,
    get_saved_language, localized, provide_i18n_context, save_language, t, t_fmt, t_or, use_i18n,
    ENGLISH_LANG_ID_STR, THAI_LANG_ID_STR,
};
pub use requestor::{HttpClient, HttpMethod, HttpResponse, RequestBuilder};
