//! i18n Models and Contracts.

pub mod i18n;

pub use i18n::*;

// Re-export Sea-ORM entity modules for direct query construction
pub use domain::entity::{db_label, db_language};
