//! Tests for the error localization middleware (Accept-Language -> `error.<CODE>` templates).

use infrastructure::middleware::{ErrorLocalizer, localize_error_responses};
use application_core::traits::{ErrorTemplateError, ErrorTemplateSource};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use infrastructure::wire::{WireFormat, from_cbor_slice, hex_decode};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

struct MockSource {
    fail: bool,
    calls: AtomicUsize,
}

#[async_trait]
impl ErrorTemplateSource for MockSource {
    async fn error_templates(
        &self,
        language_code: &str,
    ) -> Result<HashMap<String, String>, ErrorTemplateError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            return Err(ErrorTemplateError("down".to_string()));
        }
        let (in_use, plain) = match language_code {
            "en" => ("Cannot delete ({detail})", "Something went wrong"),
            _ => ("ลบไม่ได้ ({detail})", "เกิดข้อผิดพลาด"),
        };
        Ok(HashMap::from([
            ("IN_USE".to_string(), in_use.to_string()),
            ("INTERNAL_ERROR".to_string(), plain.to_string()),
        ]))
    }
}

fn error_response(wire: WireFormat, status: StatusCode, code: &str, message: &str) -> Response {
    let body = json!({ "success": false, "code": code, "message": message });
    let mut resp = wire.respond(body);
    *resp.status_mut() = status;
    resp
}

fn app_with(source: Arc<MockSource>) -> Router {
    Router::new()
        .route(
            "/in-use",
            get(|wire: WireFormat| async move {
                error_response(wire, StatusCode::CONFLICT, "IN_USE", "branch BKK-01")
            }),
        )
        .route(
            "/internal",
            get(|wire: WireFormat| async move {
                error_response(wire, StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "boom")
            }),
        )
        .route(
            "/unknown",
            get(|wire: WireFormat| async move {
                error_response(wire, StatusCode::BAD_REQUEST, "SOMETHING_ELSE", "original")
            }),
        )
        .route(
            "/ok",
            get(|| async { Json(json!({ "success": false, "code": "IN_USE", "message": "x" })) }),
        )
        .route("/plain", get(|| async { (StatusCode::BAD_REQUEST, "plain text").into_response() }))
        .layer(axum::middleware::from_fn_with_state(
            Arc::new(ErrorLocalizer::new(source)),
            localize_error_responses,
        ))
}

fn app() -> Router {
    app_with(Arc::new(MockSource { fail: false, calls: AtomicUsize::new(0) }))
}

async fn call(app: Router, uri: &str, lang: Option<&str>, accept: Option<&str>) -> (StatusCode, Response) {
    let mut req = Request::builder().uri(uri);
    if let Some(l) = lang {
        req = req.header(header::ACCEPT_LANGUAGE, l);
    }
    if let Some(a) = accept {
        req = req.header(header::ACCEPT, a);
    }
    let resp = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
    (resp.status(), resp)
}

async fn json_body(resp: Response) -> Value {
    serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap()
}

#[tokio::test]
async fn accept_language_selects_template_and_substitutes_detail() {
    let (status, resp) = call(app(), "/in-use", Some("en-US,en;q=0.9"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let body = json_body(resp).await;
    assert_eq!(body["message"], "Cannot delete (branch BKK-01)");
    assert_eq!(body["code"], "IN_USE");
    assert_eq!(body["success"], false);

    let (_, resp) = call(app(), "/in-use", Some("TH"), None).await;
    assert_eq!(json_body(resp).await["message"], "ลบไม่ได้ (branch BKK-01)");
}

#[tokio::test]
async fn template_without_detail_replaces_message() {
    let (_, resp) = call(app(), "/internal", Some("en"), None).await;
    assert_eq!(json_body(resp).await["message"], "Something went wrong");
}

#[tokio::test]
async fn unknown_or_missing_language_falls_back_to_thai() {
    for lang in [Some("fr-FR"), Some("*"), None] {
        let (_, resp) = call(app(), "/internal", lang, None).await;
        assert_eq!(json_body(resp).await["message"], "เกิดข้อผิดพลาด");
    }
}

#[tokio::test]
async fn unknown_code_is_untouched() {
    let (status, resp) = call(app(), "/unknown", Some("en"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json_body(resp).await["message"], "original");
}

#[tokio::test]
async fn success_and_non_error_bodies_are_untouched() {
    let (status, resp) = call(app(), "/ok", Some("en"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json_body(resp).await["message"], "x");

    let (status, resp) = call(app(), "/plain", Some("en"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(to_bytes(resp.into_body(), usize::MAX).await.unwrap().as_ref(), b"plain text");
}

#[tokio::test]
async fn source_failure_leaves_response_untouched() {
    let app = app_with(Arc::new(MockSource { fail: true, calls: AtomicUsize::new(0) }));
    let (status, resp) = call(app, "/in-use", Some("en"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json_body(resp).await["message"], "branch BKK-01");
}

#[tokio::test]
async fn cbor_body_is_localized_and_reencoded() {
    let (_, resp) = call(app(), "/in-use", Some("en"), Some("application/cbor")).await;
    assert_eq!(resp.headers()[header::CONTENT_TYPE], "application/cbor");
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body: Value = from_cbor_slice(&bytes).unwrap();
    assert_eq!(body["message"], "Cannot delete (branch BKK-01)");
}

#[tokio::test]
async fn cbor_hex_body_is_localized_with_correct_content_length() {
    let (_, resp) = call(app(), "/in-use", Some("th"), Some("text/cbor-hex")).await;
    assert!(resp.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/cbor-hex"));
    let declared: usize = resp.headers()[header::CONTENT_LENGTH].to_str().unwrap().parse().unwrap();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(declared, bytes.len());
    let raw = hex_decode(std::str::from_utf8(&bytes).unwrap()).unwrap();
    let body: Value = from_cbor_slice(&raw).unwrap();
    assert_eq!(body["message"], "ลบไม่ได้ (branch BKK-01)");
}

#[tokio::test]
async fn templates_are_cached_per_language() {
    let source = Arc::new(MockSource { fail: false, calls: AtomicUsize::new(0) });
    let app = app_with(source.clone());
    for _ in 0..3 {
        call(app.clone(), "/in-use", Some("en"), None).await;
    }
    call(app.clone(), "/in-use", Some("th"), None).await;
    assert_eq!(source.calls.load(Ordering::SeqCst), 2);
}
