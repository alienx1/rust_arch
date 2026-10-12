//! i18n Models: language identifiers, descriptors, and UI translation dictionary DTOs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Well-known UUIDv7 for Thai language (`db_language.id`) from `02_db_language_data.sql`.
pub const THAI_LANG_ID_STR: &str = "019ca1eb-21c0-7000-8001-000000000001";

/// Well-known UUIDv7 for English language (`db_language.id`) from `02_db_language_data.sql`.
pub const ENGLISH_LANG_ID_STR: &str = "019ca1eb-21c0-7000-8002-000000000002";

/// Metadata for a supported system language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub code: String,
    pub name: String,
    pub id: String,
}

impl LanguageInfo {
    pub fn thai() -> Self {
        Self {
            code: "th".to_string(),
            name: "ไทย".to_string(),
            id: THAI_LANG_ID_STR.to_string(),
        }
    }

    pub fn english() -> Self {
        Self {
            code: "en".to_string(),
            name: "English".to_string(),
            id: ENGLISH_LANG_ID_STR.to_string(),
        }
    }

    pub fn supported_languages() -> Vec<Self> {
        vec![Self::thai(), Self::english()]
    }
}

/// DTO representing the translated dictionary loaded from `GET /api/v1/master/localization/{lid}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nDictionaryDto {
    pub language_id: Uuid,
    pub labels: HashMap<String, String>,
}
