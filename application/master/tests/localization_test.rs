//! Integration and unit tests for Application Master Localization feature.
//! Located at `application/master/tests/` (same layer as `src`).

use application_master::features::localization::{
    CommandHandler, GetI18nCommand, GetI18nCommandHandler, GetI18nDictionaryHandler,
    GetI18nDictionaryQuery, I18nCommandExecutor, I18nDictionary, LocalizationCommands,
    LocalizationError, LocalizationQueries, QueryHandler,
};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

// --- Mock Query Service for Testing Commands ---
struct MockLocalizationQueries {
    expected_id: Uuid,
    should_fail: bool,
}

#[async_trait]
impl LocalizationQueries for MockLocalizationQueries {
    async fn get_i18n_dictionary(
        &self,
        language_id: Uuid,
    ) -> Result<I18nDictionary, LocalizationError> {
        if self.should_fail {
            return Err(LocalizationError::Database("Mock DB error".to_string()));
        }
        if language_id != self.expected_id {
            return Err(LocalizationError::LanguageNotFound(language_id.to_string()));
        }

        let mut dict = I18nDictionary::new(language_id);
        dict.insert("auth.login", "เข้าสู่ระบบ");
        dict.insert("auth.logout", "ออกจากระบบ");
        dict.insert("common.cancel", "ยกเลิก");
        Ok(dict)
    }

    async fn get_language_id_by_code(
        &self,
        code: &str,
    ) -> Result<Option<Uuid>, LocalizationError> {
        if self.should_fail {
            return Err(LocalizationError::Database("Mock DB error".to_string()));
        }
        Ok(code.eq_ignore_ascii_case("th").then_some(self.expected_id))
    }
}

#[test]
fn test_i18n_dictionary_basic_operations() {
    let lang_id = Uuid::now_v7();
    let mut dict = I18nDictionary::new(lang_id);

    assert_eq!(dict.language_id, lang_id);
    assert!(dict.is_empty());
    assert_eq!(dict.len(), 0);

    dict.insert("menu.home", "หน้าแรก");
    dict.insert("menu.settings", "ตั้งค่า");

    assert!(!dict.is_empty());
    assert_eq!(dict.len(), 2);
    assert_eq!(dict.get("menu.home"), Some("หน้าแรก"));
    assert_eq!(dict.get("menu.settings"), Some("ตั้งค่า"));
    assert_eq!(dict.get("menu.not_existing"), None);
}

#[test]
fn test_i18n_dictionary_serialization() {
    let lang_id = Uuid::now_v7();
    let mut dict = I18nDictionary::new(lang_id);
    dict.insert("app.title", "My App");

    let json = serde_json::to_string(&dict).expect("Serialization failed");
    let deserialized: I18nDictionary = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(deserialized.language_id, lang_id);
    assert_eq!(deserialized.get("app.title"), Some("My App"));
}

#[test]
fn test_localization_error_display() {
    let err_lang = LocalizationError::LanguageNotFound("lang-123".to_string());
    assert_eq!(err_lang.to_string(), "Language not found: lang-123");

    let err_label = LocalizationError::LabelNotFound("label-456".to_string());
    assert_eq!(err_label.to_string(), "Label not found: label-456");

    let err_val = LocalizationError::Validation("invalid lid".to_string());
    assert_eq!(err_val.to_string(), "Validation failed: invalid lid");

    let err_db = LocalizationError::Database("connection timeout".to_string());
    assert_eq!(err_db.to_string(), "Database error: connection timeout");
}

#[tokio::test]
async fn test_i18n_executor_success() {
    let lang_id = Uuid::now_v7();
    let mock_queries = Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    });
    let executor = I18nCommandExecutor::new(mock_queries);

    let result = executor.get_i18n(lang_id).await;
    assert!(result.is_ok());

    let dict = result.unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));
    assert_eq!(dict.get("auth.logout"), Some("ออกจากระบบ"));
}

#[tokio::test]
async fn test_i18n_executor_not_found() {
    let lang_id = Uuid::now_v7();
    let other_id = Uuid::now_v7();
    let mock_queries = Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    });
    let executor = I18nCommandExecutor::new(mock_queries);

    let result = executor.get_i18n(other_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        LocalizationError::LanguageNotFound(id) => assert_eq!(id, other_id.to_string()),
        other => panic!("Expected LanguageNotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_i18n_executor_database_error() {
    let lang_id = Uuid::now_v7();
    let mock_queries = Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: true,
    });
    let executor = I18nCommandExecutor::new(mock_queries);

    let result = executor.get_i18n(lang_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        LocalizationError::Database(msg) => assert_eq!(msg, "Mock DB error"),
        other => panic!("Expected Database error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_get_i18n_command_handler() {
    let lang_id = Uuid::now_v7();
    let mock_queries = Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    });
    let executor: Arc<dyn LocalizationCommands> = Arc::new(I18nCommandExecutor::new(mock_queries));
    let handler = GetI18nCommandHandler::new(executor);

    let cmd = GetI18nCommand::new(lang_id);
    let result = handler.handle(cmd).await;
    assert!(result.is_ok());
    let dict = result.unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("common.cancel"), Some("ยกเลิก"));
}

#[tokio::test]
async fn test_get_i18n_dictionary_query_handler() {
    let lang_id = Uuid::now_v7();
    let mock_queries = Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    });
    let handler = GetI18nDictionaryHandler::new(mock_queries);

    let query = GetI18nDictionaryQuery::new(lang_id);
    let result = handler.handle(query).await;
    assert!(result.is_ok());
    let dict = result.unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));
}

#[tokio::test]
async fn test_i18n_executor_by_code() {
    let lang_id = Uuid::now_v7();
    let executor = I18nCommandExecutor::new(Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    }));

    let dict = executor.get_i18n_by_code("TH").await.unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));

    match executor.get_i18n_by_code("xx").await.unwrap_err() {
        LocalizationError::LanguageNotFound(code) => assert_eq!(code, "xx"),
        other => panic!("Expected LanguageNotFound, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_error_template_source_filters_error_category() {
    use application_master::features::localization::{DbErrorTemplateSource, ErrorTemplateSource};

    let lang_id = Uuid::now_v7();
    let source = DbErrorTemplateSource::new(Arc::new(MockLocalizationQueries {
        expected_id: lang_id,
        should_fail: false,
    }));
    let templates = source.error_templates("th").await.unwrap();
    assert!(templates.keys().all(|k| !k.starts_with("error.")));
    assert!(source.error_templates("xx").await.is_err());
}
