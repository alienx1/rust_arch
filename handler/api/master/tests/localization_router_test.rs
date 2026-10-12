//! Integration tests for Localization (i18n) router endpoints.

use api_master::routers::localization_routes_with_commands;
use application_master::features::localization::{
    I18nDictionary, LocalizationCommands, LocalizationError,
};
use async_trait::async_trait;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

struct MockLocalizationCommands;

#[async_trait]
impl LocalizationCommands for MockLocalizationCommands {
    async fn get_i18n(&self, language_id: Uuid) -> Result<I18nDictionary, LocalizationError> {
        if language_id == Uuid::nil() {
            return Err(LocalizationError::LanguageNotFound(language_id.to_string()));
        }
        let mut dict = I18nDictionary::new(language_id);
        dict.insert("auth.login", "เข้าสู่ระบบ");
        dict.insert("common.cancel", "ยกเลิก");
        Ok(dict)
    }

    async fn get_i18n_by_code(&self, code: &str) -> Result<I18nDictionary, LocalizationError> {
        match code.to_lowercase().as_str() {
            "th" => self.get_i18n(Uuid::now_v7()).await,
            _ => Err(LocalizationError::LanguageNotFound(code.to_string())),
        }
    }
}

fn test_app() -> Router {
    let mock = Arc::new(MockLocalizationCommands);
    localization_routes_with_commands(mock)
}

#[tokio::test]
async fn test_get_i18n_by_path_success() {
    let app = test_app();
    let lang_id = Uuid::now_v7();

    let req = Request::builder()
        .uri(format!("/{lang_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dict: I18nDictionary = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));
    assert_eq!(dict.get("common.cancel"), Some("ยกเลิก"));
}

#[tokio::test]
async fn test_get_i18n_by_path_alias() {
    let app = test_app();
    let lang_id = Uuid::now_v7();

    let req = Request::builder()
        .uri(format!("/i18n/{lang_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_i18n_by_query_lid() {
    let app = test_app();
    let lang_id = Uuid::now_v7();

    let req = Request::builder()
        .uri(format!("/?lid={lang_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dict: I18nDictionary = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(dict.language_id, lang_id);
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));
}

#[tokio::test]
async fn test_get_i18n_by_query_language_id_alias() {
    let app = test_app();
    let lang_id = Uuid::now_v7();

    let req = Request::builder()
        .uri(format!("/?language_id={lang_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_i18n_missing_query_param() {
    let app = test_app();

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err["success"], false);
    assert_eq!(err["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_get_i18n_not_found() {
    let app = test_app();
    let nil_id = Uuid::nil();

    let req = Request::builder()
        .uri(format!("/{nil_id}"))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(err["success"], false);
    assert_eq!(err["code"], "LANGUAGE_NOT_FOUND");
}

#[tokio::test]
async fn test_get_i18n_invalid_uuid() {
    let app = test_app();

    let req = Request::builder()
        .uri("/not-a-valid-uuid")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_i18n_by_code_success_case_insensitive() {
    let req = Request::builder().uri("/code/TH").body(Body::empty()).unwrap();
    let resp = test_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dict: I18nDictionary = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(dict.get("auth.login"), Some("เข้าสู่ระบบ"));
}

#[tokio::test]
async fn test_get_i18n_by_code_unknown_language() {
    let req = Request::builder().uri("/code/xx").body(Body::empty()).unwrap();
    let resp = test_app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["code"], "LANGUAGE_NOT_FOUND");
}
