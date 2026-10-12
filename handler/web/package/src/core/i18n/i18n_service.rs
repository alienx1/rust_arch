//! i18n HTTP fetcher service: retrieves dictionary mappings from `db_label` endpoints.

use crate::core::requestor::HttpClient;
use super::i18n_models::{I18nDictionaryDto, ENGLISH_LANG_ID_STR, THAI_LANG_ID_STR};

/// Service responsible for fetching i18n dictionary from API backend.
#[derive(Clone, Debug)]
pub struct I18nService {
    http: HttpClient,
    api_base: String,
}

impl Default for I18nService {
    fn default() -> Self {
        Self::new("/api/v1/master/localization")
    }
}

impl I18nService {
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            http: HttpClient::default(),
            api_base: api_base.into(),
        }
    }

    /// Maps language code ("th", "en") to database UUID string.
    pub fn lang_id_for_code(code: &str) -> &'static str {
        match code.trim().to_lowercase().as_str() {
            "en" => ENGLISH_LANG_ID_STR,
            _ => THAI_LANG_ID_STR,
        }
    }

    /// Fetches translation dictionary by language code or UUID string.
    pub async fn fetch_dictionary(&self, code_or_uuid: &str) -> Result<I18nDictionaryDto, String> {
        let lid = if code_or_uuid.len() == 36 && code_or_uuid.contains('-') {
            code_or_uuid
        } else {
            Self::lang_id_for_code(code_or_uuid)
        };

        let target_url = format!("{}/{}", self.api_base, lid);
        match self.http.get(&target_url).send().await {
            Ok(res) if res.is_success() => res
                .auto::<I18nDictionaryDto>()
                .map_err(|e| format!("Failed to parse i18n dictionary: {e}")),
            Ok(res) => {
                // Secondary fallback attempt against /api/v1/localization
                let fallback_url = format!("/api/v1/localization/{}", lid);
                if let Ok(res2) = self.http.get(&fallback_url).send().await {
                    if res2.is_success() {
                        return res2
                            .auto::<I18nDictionaryDto>()
                            .map_err(|e| format!("Failed to parse fallback i18n: {e}"));
                    }
                }
                Err(format!("Localization endpoint HTTP status {}", res.status()))
            }
            Err(e) => {
                // Attempt fallback
                let fallback_url = format!("/api/v1/localization/{}", lid);
                if let Ok(res2) = self.http.get(&fallback_url).send().await {
                    if res2.is_success() {
                        return res2
                            .auto::<I18nDictionaryDto>()
                            .map_err(|e| format!("Failed to parse fallback i18n: {e}"));
                    }
                }
                Err(format!("Network error while fetching i18n dictionary: {e}"))
            }
        }
    }
}
