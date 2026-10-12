//! i18n Models for frontend UI dictionary translation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Label entity model directly from Sea-ORM entity `db_label`.
pub type Label = domain::entity::db_label::Model;

/// Language entity model directly from Sea-ORM entity `db_language`.
pub type Language = domain::entity::db_language::Model;

/// i18n Translation Dictionary returned to the frontend.
///
/// Contains all key-value mappings formatted as `"<category>.<key>"` => `"<value>"`
/// for the specified `language_id` (lid of `db_language`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nDictionary {
    /// Language ID (`db_language.id` / lid)
    pub language_id: Uuid,
    /// Translations map: `"<category>.<key>"` -> `"<localized string>"`
    pub labels: HashMap<String, String>,
}

impl I18nDictionary {
    pub fn new(language_id: Uuid) -> Self {
        Self {
            language_id,
            labels: HashMap::new(),
        }
    }

    pub fn with_capacity(language_id: Uuid, capacity: usize) -> Self {
        Self {
            language_id,
            labels: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, full_key: impl Into<String>, value: impl Into<String>) {
        self.labels.insert(full_key.into(), value.into());
    }

    pub fn get(&self, full_key: &str) -> Option<&str> {
        self.labels.get(full_key).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}
