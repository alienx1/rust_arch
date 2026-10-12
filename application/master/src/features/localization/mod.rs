//! # Localization Feature Scope
//!
//! **Scope & Purpose:**
//! This module is dedicated **exclusively** for frontend `i18n` (internationalization) consumption.
//! It provides:
//! - **Commands Interface ([`LocalizationCommands`]):** Trait contract called by API routes in `handler/api/master`.
//! - **Queries Interface ([`LocalizationQueries`]):** Trait contract and Sea-ORM service reading from `db_label`.
//! - **Models ([`I18nDictionary`]):** Mappings of `"<category>.<key>"` => `"<value>"` for the specified `language_id` (lid).
//!
//! **API Endpoints:**
//! Routes accept `lid` (`language_id` of `db_language`, e.g. `Uuid`) and invoke `commands.get_i18n(lid)`,
//! or a language code (`th`, `en`) and invoke `commands.get_i18n_by_code(code)`.
//!
//! **Error templates:** [`ErrorTemplateSource`] exposes the `error.<CODE>` labels for the
//! API error-localization middleware.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;
/// Language of the request being served (lives in `application-core`, shared with the middleware)
pub use application_core::request_language;

pub use commands::*;
pub use errors::*;
pub use models::*;
pub use queries::*;
