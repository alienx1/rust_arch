//! Tests for the audit trail middleware: what is recorded, masked, converted and skipped.

use infrastructure::middleware::audit_body::{BODY_STORE_MAX, body_to_text, cbor_to_json, headers_to_json, mask_json};
use infrastructure::middleware::{AuditTrailState, audit_trail, derive_audit_target, should_audit};
use application_core::traits::{AuditError, AuditEvent, AuditLogPort, AuditPage, AuditSearch};
use async_trait::async_trait;
use axum::body::{Body, Bytes, to_bytes};
use axum::extract::Request as AxumRequest;
use axum::http::{Request, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use infrastructure::middleware::AuthenticatedUser;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower::ServiceExt;

#[derive(Default)]
struct CapturingLog(Mutex<Vec<AuditEvent>>);

#[async_trait]
impl AuditLogPort for CapturingLog {
    fn record(&self, events: Vec<AuditEvent>) {
        self.0.lock().unwrap().extend(events);
    }
    async fn search(&self, f: AuditSearch) -> Result<AuditPage, AuditError> {
        Ok(AuditPage { page: f.page, limit: f.limit, ..Default::default() })
    }
    async fn get(&self, _id: &str) -> Result<Option<AuditEvent>, AuditError> {
        Ok(None)
    }
    async fn is_available(&self) -> bool {
        true
    }
}

impl CapturingLog {
    /// The middleware records from a spawned task: wait for it
    async fn events(&self, expected: usize) -> Vec<AuditEvent> {
        for _ in 0..100 {
            if self.0.lock().unwrap().len() >= expected {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        self.0.lock().unwrap().clone()
    }
    /// Gives a (wrongly) recorded event time to show up, then returns how many there are
    async fn count_after_grace(&self) -> usize {
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.0.lock().unwrap().len()
    }
}

async fn echo_len(body: Bytes) -> impl IntoResponse {
    Json(json!({ "received": body.len(), "token": "abc", "nested": { "Password": "p" } }))
}

fn app(log: Arc<CapturingLog>) -> Router {
    let port: Arc<dyn AuditLogPort> = log;
    Router::new()
        .route("/api/v1/db/dbpg004/items", post(echo_len).get(|| async { Json(json!({"ok": true})) }))
        .route("/api/v1/db/dbpg004/missing", get(|| async { StatusCode::NOT_FOUND }))
        .route("/api/v1/su/supg004/users/{id}/recovery-link", post(|| async { "ok" }))
        .route("/api/v1/audit/events", post(|| async { "ok" }))
        .route("/api/v1/al/alpg000/events", any(|| async { "ok" }))
        .route("/api/v1/i18n/{lid}", get(|| async { "ok" }))
        .route("/api/v1/master/health", get(|| async { "ok" }))
        .route("/health", get(|| async { "ok" }))
        .route("/api/v1/sleepy", get(|| async {
            tokio::time::sleep(Duration::from_millis(40)).await;
            "zzz"
        }))
        .route("/api/v1/cbor", post(|| async {
            let mut buf = Vec::new();
            ciborium::into_writer(&json!({"id": "x", "secret": "s"}), &mut buf).unwrap();
            ([(header::CONTENT_TYPE, "application/cbor")], buf)
        }))
        .route("/api/v1/big", get(|| async { "x".repeat(300 * 1024) }))
        .route("/api/v1/long", get(|| async { Json(json!({"text": "é".repeat(20_000)})) }))
        .route("/api/v1/su/supg001/me", get(|| async { "me" }).layer(axum::middleware::from_fn(
            |req: AxumRequest, next: Next| async move {
                // What require_kratos_session does: user goes into the RESPONSE extensions
                let mut resp: Response = next.run(req).await;
                resp.extensions_mut().insert(AuthenticatedUser {
                    identity_id: "019a0000-0000-7000-8000-0000000000aa".into(),
                    session_id: "sess-1".into(),
                    email: Some("alice@example.com".into()),
                    traits: json!({"username": "alice"}),
                });
                resp
            },
        )))
        .layer(axum::middleware::from_fn_with_state(AuditTrailState::new(port, None), audit_trail))
}

async fn send(router: &Router, req: Request<Body>) -> Response {
    router.clone().oneshot(req).await.unwrap()
}

fn json_post(uri: &str, body: &str) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, body.len())
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn as_json(text: &Option<String>) -> Value {
    serde_json::from_str(text.as_deref().expect("body recorded")).expect("body is JSON")
}

#[tokio::test]
async fn records_request_fields_and_masks_secrets() {
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());
    let payload = r#"{"name":"a","password":"hunter2","deep":{"apiKey":"k"}}"#;
    let req = Request::post("/api/v1/db/dbpg004/items?page=2&token=abc")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, payload.len())
        .header("cookie", "ory_kratos_session=SECRET")
        .header("authorization", "Bearer SECRET")
        .header("x-api-key", "SECRET")
        .header("x-session-token", "SECRET")
        .header("x-forwarded-for", "203.0.113.9, 10.0.0.1")
        .header("accept-language", "th-TH,th;q=0.9,en;q=0.8")
        .header("user-agent", "test-agent/1.0")
        .body(Body::from(payload))
        .unwrap();
    let resp = send(&router, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    // The handler still received the whole body
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["received"], payload.len());

    let events = log.events(1).await;
    assert_eq!(events.len(), 1);
    let e = &events[0];
    assert_eq!(e.kind, "api");
    assert_eq!(e.method.as_deref(), Some("POST"));
    assert_eq!(e.path.as_deref(), Some("/api/v1/db/dbpg004/items"));
    assert_eq!(e.query.as_deref(), Some("page=2&token=***"));
    assert_eq!(e.status, Some(200));
    assert!(e.duration_ms.is_some());
    assert_eq!(e.program_code.as_deref(), Some("DBPG004"));
    assert_eq!(e.action.as_deref(), Some("CREATE"));
    assert_eq!(e.ip.as_deref(), Some("203.0.113.9"));
    assert_eq!(e.language.as_deref(), Some("th"));
    assert_eq!(e.user_agent.as_deref(), Some("test-agent/1.0"));
    assert!(uuid::Uuid::parse_str(&e.event_id).is_ok());
    assert!(chrono::DateTime::parse_from_rfc3339(&e.timestamp).is_ok());
    assert!(e.user_id.is_none(), "no session layer, no user");

    let h = e.request_headers.as_ref().unwrap();
    for name in ["cookie", "authorization", "x-api-key", "x-session-token"] {
        assert_eq!(h[name], "***", "{name}");
    }
    assert_eq!(h["user-agent"], "test-agent/1.0");
    assert_eq!(h["content-type"], "application/json");

    let req_body = as_json(&e.request_body);
    assert_eq!(req_body["name"], "a");
    assert_eq!(req_body["password"], "***");
    assert_eq!(req_body["deep"]["apiKey"], "***");
    let res_body = as_json(&e.response_body);
    assert_eq!(res_body["received"], payload.len());
    assert_eq!(res_body["token"], "***");
    assert_eq!(res_body["nested"]["Password"], "***");
}

#[tokio::test]
async fn cbor_bodies_become_json_with_uuid_strings() {
    let id = uuid::Uuid::now_v7();
    let value = ciborium::Value::Map(vec![
        (ciborium::Value::Text("id".into()), ciborium::Value::Bytes(id.as_bytes().to_vec())),
        (ciborium::Value::Text("blob".into()), ciborium::Value::Bytes(vec![1, 2, 3])),
        (ciborium::Value::Text("n".into()), ciborium::Value::Integer(7.into())),
        (ciborium::Value::Text("password".into()), ciborium::Value::Text("x".into())),
    ]);
    let mut raw = Vec::new();
    ciborium::into_writer(&value, &mut raw).unwrap();

    let text = body_to_text(Some("application/cbor"), &raw).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["id"], id.to_string());
    assert_eq!(v["blob"], "<binary 3 bytes>");
    assert_eq!(v["n"], 7);
    assert_eq!(v["password"], "***");

    let hex: String = raw.iter().map(|b| format!("{b:02x}")).collect();
    let from_hex = body_to_text(Some("text/cbor-hex; charset=utf-8"), hex.as_bytes()).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&from_hex).unwrap(), v);

    assert_eq!(body_to_text(Some("application/cbor"), &[0xff, 0xff, 0xff]).unwrap(), "<binary 3 bytes>");
    assert_eq!(cbor_to_json(&ciborium::Value::Null), Value::Null);

    // End to end: a CBOR response is stored as JSON
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());
    let req = Request::post("/api/v1/cbor")
        .header(header::CONTENT_TYPE, "application/cbor")
        .header(header::CONTENT_LENGTH, raw.len())
        .body(Body::from(raw))
        .unwrap();
    send(&router, req).await;
    let e = &log.events(1).await[0];
    assert_eq!(as_json(&e.request_body)["id"], id.to_string());
    let res = as_json(&e.response_body);
    assert_eq!(res["id"], "x");
    assert_eq!(res["secret"], "***");
}

#[test]
fn other_content_types() {
    assert_eq!(body_to_text(Some("text/plain"), b"hello").unwrap(), "hello");
    assert_eq!(body_to_text(None, b"").as_ref(), None);
    assert_eq!(body_to_text(Some("application/pdf"), b"%PDF").unwrap(), "<binary 4 bytes>");
    assert_eq!(body_to_text(Some("text/plain"), &[0xff, 0xfe]).unwrap(), "<binary 2 bytes>");
    assert_eq!(
        body_to_text(Some("application/x-www-form-urlencoded"), b"a=1&password=x").unwrap(),
        "a=1&password=***"
    );
    // JSON without a content type is still masked
    assert_eq!(body_to_text(None, br#"{"token":"t","a":1}"#).unwrap(), r#"{"a":1,"token":"***"}"#);
    let mut v = json!([{"cookie": "c", "Credential": 1, "api-key": 2, "keep": 3}]);
    mask_json(&mut v);
    assert_eq!(v, json!([{"cookie": "***", "Credential": "***", "api-key": "***", "keep": 3}]));
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-custom-token", "t".parse().unwrap());
    headers.insert("x-webhook-secret", "t".parse().unwrap());
    headers.insert("accept", "a".parse().unwrap());
    headers.append("accept", "b".parse().unwrap());
    let j = headers_to_json(&headers);
    assert_eq!(j["x-custom-token"], "***");
    assert_eq!(j["x-webhook-secret"], "***");
    assert_eq!(j["accept"], "a, b");
}

#[tokio::test]
async fn long_bodies_are_truncated_and_huge_ones_not_stored() {
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());

    send(&router, Request::get("/api/v1/long").body(Body::empty()).unwrap()).await;
    let e = &log.events(1).await[0];
    let stored = e.response_body.as_deref().unwrap();
    assert!(stored.ends_with("…(truncated)"));
    assert!(stored.len() <= BODY_STORE_MAX + "…(truncated)".len());

    let resp = send(&router, Request::get("/api/v1/big").body(Body::empty()).unwrap()).await;
    // The client still gets all of it
    assert_eq!(to_bytes(resp.into_body(), usize::MAX).await.unwrap().len(), 300 * 1024);
    let e = &log.events(2).await[1];
    assert_eq!(e.response_body.as_deref(), Some("<307200 bytes, not stored>"));

    // A request over the cap reaches the handler intact
    let big = "y".repeat(300 * 1024);
    let resp = send(&router, json_post("/api/v1/db/dbpg004/items", &big)).await;
    let v: Value = serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(v["received"], 300 * 1024);
    let e = &log.events(3).await[2];
    assert_eq!(e.request_body.as_deref(), Some("<307200 bytes, not stored>"));
}

#[tokio::test]
async fn skipped_requests_record_nothing() {
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());
    for (method, uri) in [
        ("POST", "/api/v1/audit/events"),
        ("GET", "/api/v1/al/alpg000/events"),
        ("GET", "/api/v1/i18n/th"),
        ("GET", "/api/v1/master/health"),
        ("GET", "/health"),
        ("OPTIONS", "/api/v1/db/dbpg004/items"),
    ] {
        send(&router, Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()).await;
    }
    assert_eq!(log.count_after_grace().await, 0);

    // The audit program's own writes are still recorded
    send(&router, Request::post("/api/v1/al/alpg000/events").body(Body::empty()).unwrap()).await;
    assert_eq!(log.events(1).await.len(), 1);
}

#[tokio::test]
async fn status_duration_and_failures_are_recorded() {
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());
    let resp = send(&router, Request::get("/api/v1/db/dbpg004/missing").body(Body::empty()).unwrap()).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    send(&router, Request::get("/api/v1/sleepy").body(Body::empty()).unwrap()).await;
    send(&router, Request::get("/api/v1/does-not-exist").body(Body::empty()).unwrap()).await;

    let events = log.events(3).await;
    let find = |p: &str| events.iter().find(|e| e.path.as_deref() == Some(p)).unwrap().clone();
    let missing = find("/api/v1/db/dbpg004/missing");
    assert_eq!((missing.status, missing.action.as_deref()), (Some(404), Some("VIEW")));
    assert!(find("/api/v1/sleepy").duration_ms.unwrap() >= 40);
    assert_eq!(find("/api/v1/does-not-exist").status, Some(404));
}

#[tokio::test]
async fn user_comes_from_the_response_extensions() {
    let log = Arc::new(CapturingLog::default());
    let router = app(log.clone());
    send(&router, Request::get("/api/v1/su/supg001/me").body(Body::empty()).unwrap()).await;
    let e = &log.events(1).await[0];
    assert_eq!(e.user_id.as_deref(), Some("019a0000-0000-7000-8000-0000000000aa"));
    assert_eq!(e.username.as_deref(), Some("alice"));
    assert_eq!(e.email.as_deref(), Some("alice@example.com"));
    assert_eq!(e.session_id.as_deref(), Some("sess-1"));
    assert_eq!(e.program_code.as_deref(), Some("SUPG001"));
}

#[test]
fn derive_audit_target_table() {
    let cases: [(&str, &str, Option<&str>, &str); 17] = [
        ("GET", "/api/v1/db/dbpg004/items", Some("DBPG004"), "VIEW"),
        ("POST", "/api/v1/db/dbpg004/items", Some("DBPG004"), "CREATE"),
        ("PUT", "/api/v1/db/dbpg004/items/1", Some("DBPG004"), "UPDATE"),
        ("DELETE", "/api/v1/db/dbpg004/items/1", Some("DBPG004"), "DELETE"),
        ("GET", "/api/v1/master/su/supg001/users", Some("SUPG001"), "VIEW"),
        ("POST", "/api/v1/su/supg004/users/9/recovery-link", Some("SUPG004"), "RECOVERY_LINK"),
        ("PUT", "/api/v1/su/supg003/groups/9/matrix", Some("SUPG003"), "UPDATE_MATRIX"),
        ("DELETE", "/api/v1/su/supg004/sessions/abc", Some("SUPG004"), "REVOKE"),
        ("DELETE", "/api/v1/su/supg004/users/9/sessions", Some("SUPG004"), "REVOKE"),
        ("POST", "/api/v1/su/supg004/users/9/suspend", Some("SUPG004"), "SUSPEND"),
        ("POST", "/api/v1/su/supg004/users/9/reactivate", Some("SUPG004"), "REACTIVATE"),
        ("POST", "/api/v1/al/alpg000/events", Some("ALPG000"), "CREATE"),
        ("GET", "/api/v1/dashboard/summary", Some("SYPG000"), "VIEW"),
        ("POST", "/api/v1/hooks/login-guard", None, "LOGIN"),
        ("PATCH", "/api/v1/db/dbpg002/x", Some("DBPG002"), "UPDATE"),
        ("GET", "/api/v1/something/else", None, "VIEW"),
        ("get", "/api/v1/db/DBPG001/x", Some("DBPG001"), "VIEW"),
    ];
    for (method, path, program, action) in cases {
        assert_eq!(
            derive_audit_target(method, path),
            (program.map(str::to_string), action.to_string()),
            "{method} {path}"
        );
    }
}

#[test]
fn should_audit_table() {
    for (method, path, expected) in [
        ("GET", "/api/v1/db/dbpg004/items", true),
        ("POST", "/api/v1/hooks/login-guard", true),
        ("POST", "/api/v1/master/su/supg001/users", true),
        ("POST", "/api/v1/audit/events", false),
        ("GET", "/api/v1/audit/anything", false),
        ("GET", "/api/v1/al/alpg000/events", false),
        ("HEAD", "/api/v1/al/alpg000/events", false),
        ("POST", "/api/v1/al/alpg000/events", true),
        ("GET", "/api/v1/i18n/th", false),
        ("GET", "/api/v1/master/localization/th", false),
        ("GET", "/api/v1/master/health", false),
        ("GET", "/health", false),
        ("GET", "/", false),
        ("GET", "/assets/app.js", false),
        ("OPTIONS", "/api/v1/db/dbpg004/items", false),
    ] {
        assert_eq!(should_audit(method, path), expected, "{method} {path}");
    }
}
