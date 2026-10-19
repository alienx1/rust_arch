//! The sign-in side of the password policy: the Kratos hooks (inactive account, forced password
//! change, password changed) and the gateway that locks an account after too many failed sign-ins.
//! Located at `handler/api/master/tests/password_policy_router_test.rs`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use api_master::routers::{
    GatewayState, HookState, auth_gateway_routes_with_state, kratos_hook_routes_with_state,
};
use application_core::traits::{
    IdentityError, IdentityProvisioningPort, InMemoryCache, MailError, MailMessage, MailService, NewIdentity,
    RecoveryLink,
};
use application_master::features::su::account_security::queries::AccountRef;
use application_master::features::su::account_security::{
    AccountSecurityCommands, AccountSecurityError, AccountSecurityExecutor, AccountSecurityQueries,
};
use application_master::features::su::supg005::{
    Supg005CommandExecutor, Supg005Commands, Supg005Error, Supg005Queries,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::extract::Query;
use axum::http::{Request, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use tower::ServiceExt;
use uuid::Uuid;

// ---------- the account side (in memory)

#[derive(Default)]
struct Rows {
    accounts: Vec<AccountRef>,
}

#[derive(Default)]
struct MemoryQueries(Mutex<Rows>);

#[async_trait]
impl AccountSecurityQueries for MemoryQueries {
    async fn find_account(&self, id: Uuid) -> Result<Option<AccountRef>, AccountSecurityError> {
        Ok(self.0.lock().unwrap().accounts.iter().find(|a| a.id == id).cloned())
    }
    async fn find_account_by_identifier(&self, identifier: &str) -> Result<Option<AccountRef>, AccountSecurityError> {
        let name = identifier.split('@').next().unwrap().to_lowercase();
        Ok(self.0.lock().unwrap().accounts.iter().find(|a| a.username == name).cloned())
    }
    async fn set_must_change_password(&self, id: Uuid, value: bool) -> Result<bool, AccountSecurityError> {
        let mut rows = self.0.lock().unwrap();
        match rows.accounts.iter_mut().find(|a| a.id == id) {
            Some(a) => {
                a.must_change_password = value;
                Ok(true)
            }
            None => Ok(false),
        }
    }
    async fn security_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError> {
        Ok(HashMap::new()) // the defaults: 5 failures, 15 minutes
    }
}

struct Identity;

#[async_trait]
impl IdentityProvisioningPort for Identity {
    async fn create_identity(&self, _i: NewIdentity) -> Result<String, IdentityError> {
        unreachable!()
    }
    async fn delete_identity(&self, _id: &str) -> Result<(), IdentityError> {
        unreachable!()
    }
    async fn create_recovery_link(&self, identifier: &str) -> Result<RecoveryLink, IdentityError> {
        Ok(RecoveryLink { url: format!("http://localhost:3000/recovery?user={identifier}"), expires_at: None })
    }
    async fn find_identity_id(&self, _i: &str) -> Result<Option<String>, IdentityError> {
        Ok(Some("identity".into()))
    }
    async fn find_username(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        // Whose sign-in is this: the e-mail was changed, so it no longer starts with the user name
        Ok((identifier == "someone.else@arch.local").then(|| "newbie".to_string()))
    }
    async fn find_email(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        Ok(Some(format!("{identifier}@arch.local")))
    }
}

#[derive(Default)]
struct Outbox(Mutex<Vec<MailMessage>>);

#[async_trait]
impl MailService for Outbox {
    async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        self.0.lock().unwrap().push(message);
        Ok(())
    }
}

struct Security {
    exec: Arc<AccountSecurityExecutor>,
    queries: Arc<MemoryQueries>,
    outbox: Arc<Outbox>,
}

fn security() -> Security {
    let queries = Arc::new(MemoryQueries::default());
    let account = |name: &str, must: bool| AccountRef { id: Uuid::now_v7(), username: name.into(), active: true, must_change_password: must };
    queries.0.lock().unwrap().accounts = vec![account("somchai", false), account("newbie", true)];
    let outbox = Arc::new(Outbox::default());
    let exec = Arc::new(
        AccountSecurityExecutor::new(queries.clone(), Arc::new(InMemoryCache::new()))
            .with_identity(Arc::new(Identity))
            .with_mail(outbox.clone()),
    );
    Security { exec, queries, outbox }
}

// ---------- the hooks

struct IamQueries;

#[async_trait]
impl Supg005Queries for IamQueries {
    async fn list_users_with_sessions(&self) -> Result<Vec<application_master::features::su::supg005::UserSessionOverviewDto>, Supg005Error> {
        unreachable!()
    }
    async fn list_users_paginated(
        &self,
        _q: application_master::features::su::supg005::models::UserPaginationQuery,
    ) -> Result<application_master::features::su::supg005::models::PaginatedResult<application_master::features::su::supg005::UserSessionOverviewDto>, Supg005Error> {
        unreachable!()
    }
    async fn set_user_active(&self, _u: &str, _a: bool) -> Result<bool, Supg005Error> {
        unreachable!()
    }
    async fn is_login_allowed(&self, identifier: &str) -> Result<bool, Supg005Error> {
        Ok(identifier != "suspended@arch.local")
    }
}

fn hooks(security: &Security) -> Router {
    // SAFETY: every test of this file wants the same value
    unsafe { std::env::set_var("KRATOS_WEBHOOK_SECRET", "test-secret") };
    let iam: Arc<dyn Supg005Commands> = Arc::new(Supg005CommandExecutor::new(Arc::new(IamQueries)));
    kratos_hook_routes_with_state(HookState { supg005: iam, security: Some(security.exec.clone() as Arc<dyn AccountSecurityCommands>) })
}

fn hook(path: &str, identifier: &str, secret: Option<&str>) -> Request<Body> {
    let mut req = Request::builder().method("POST").uri(path).header("content-type", "application/json");
    if let Some(s) = secret {
        req = req.header("x-webhook-secret", s);
    }
    req.body(Body::from(format!(r#"{{"identifier":"{identifier}"}}"#))).unwrap()
}

async fn text(resp: axum::response::Response) -> String {
    String::from_utf8(to_bytes(resp.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap()
}

#[tokio::test]
async fn test_hooks_need_the_shared_secret() {
    let s = security();
    let app = hooks(&s);
    assert_eq!(app.clone().oneshot(hook("/login-guard", "somchai", None)).await.unwrap().status(), StatusCode::UNAUTHORIZED);
    assert_eq!(app.clone().oneshot(hook("/login-guard", "somchai", Some("wrong"))).await.unwrap().status(), StatusCode::UNAUTHORIZED);
    assert_eq!(app.clone().oneshot(hook("/password-changed", "somchai", None)).await.unwrap().status(), StatusCode::UNAUTHORIZED);
    assert_eq!(app.oneshot(hook("/login-guard", "somchai", Some("test-secret"))).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_guard_still_refuses_an_inactive_account() {
    let s = security();
    let resp = hooks(&s).oneshot(hook("/login-guard", "suspended@arch.local", Some("test-secret"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert!(text(resp).await.contains("inactive"));
}

#[tokio::test]
async fn test_an_account_that_must_change_its_password_cannot_sign_in_and_is_mailed_the_link() {
    let s = security();
    let app = hooks(&s);
    let resp = app.clone().oneshot(hook("/login-guard", "newbie@arch.local", Some("test-secret"))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "the right password is not enough");
    let body = text(resp).await;
    assert!(body.contains("4010002") && body.contains("รหัสผ่าน") && body.contains("password"), "the page says why, in both languages: {body}");

    // The mail goes out in the background; wait for it
    for _ in 0..50 {
        if !s.outbox.0.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let sent = s.outbox.0.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "newbie@arch.local");
    assert!(sent[0].text.contains("recovery?user=newbie"));

    // Still blocked on the next attempt, and not mailed again within the interval
    let again = app.clone().oneshot(hook("/login-guard", "newbie", Some("test-secret"))).await.unwrap();
    assert_eq!(again.status(), StatusCode::FORBIDDEN);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(s.outbox.0.lock().unwrap().len(), 1);

    // Setting the new password (Kratos calls the settings hook) lets the account in
    assert_eq!(app.clone().oneshot(hook("/password-changed", "newbie@arch.local", Some("test-secret"))).await.unwrap().status(), StatusCode::OK);
    assert_eq!(app.oneshot(hook("/login-guard", "newbie@arch.local", Some("test-secret"))).await.unwrap().status(), StatusCode::OK);
    assert!(!s.queries.0.lock().unwrap().accounts[1].must_change_password);
}

#[tokio::test]
async fn test_the_hook_finds_the_account_by_user_name_when_the_email_no_longer_matches_it() {
    // The e-mail was changed through the employee's contacts: its local part is not the user name
    let s = security();
    let app = hooks(&s);
    let body = |id: &str, user: &str| {
        Request::builder()
            .method("POST")
            .uri("/login-guard")
            .header("content-type", "application/json")
            .header("x-webhook-secret", "test-secret")
            .body(Body::from(format!(r#"{{"identifier":"{id}","username":"{user}"}}"#)))
            .unwrap()
    };
    let resp = app.clone().oneshot(body("someone.else@arch.local", "newbie")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "newbie must still change the password");
    assert!(text(resp).await.contains("4010002"));

    // A Kratos that does not send the user name yet (config not applied): the account is found
    // through the identity provider, not guessed from the e-mail
    let no_name = Request::builder()
        .method("POST")
        .uri("/login-guard")
        .header("content-type", "application/json")
        .header("x-webhook-secret", "test-secret")
        .body(Body::from(r#"{"identifier":"someone.else@arch.local"}"#))
        .unwrap();
    let resp = app.oneshot(no_name).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "still guarded without the user name in the hook body");
}

// ---------- the gateway, against a fake Kratos

#[derive(Clone, Default)]
struct FakeKratos {
    /// Password submits that reached Kratos
    submits: Arc<AtomicUsize>,
    /// Settings (new password) submits that reached Kratos
    settings_submits: Arc<AtomicUsize>,
}

fn flow(messages: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "id": "f1", "type": "browser", "ui": { "action": "/self-service/login?flow=f1", "method": "POST", "messages": messages, "nodes": [] } })
}

async fn fake_kratos() -> (String, FakeKratos) {
    let state = FakeKratos::default();
    let app = Router::new()
        .route(
            "/self-service/login/browser",
            get(|| async { ([(header::SET_COOKIE, "csrf_token=abc; Path=/")], Json(flow(serde_json::json!([])))).into_response() }),
        )
        .route("/self-service/login/flows", get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(q.get("id").map(String::as_str), Some("f1"));
            Json(flow(serde_json::json!([])))
        }))
        .route(
            "/self-service/login",
            post({
                let state = state.clone();
                move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                    let state = state.clone();
                    async move {
                        state.submits.fetch_add(1, Ordering::SeqCst);
                        let text = String::from_utf8_lossy(&body).to_string();
                        // JSON or form: the password is whatever follows "password"
                        let right = text.contains("\"password\":\"right\"") || text.contains("password=right");
                        let refused_by_hook = text.contains("hookrefused");
                        let _ = headers;
                        if right && !refused_by_hook {
                            (
                                StatusCode::OK,
                                [(header::SET_COOKIE, "ory_kratos_session=s1; Path=/")],
                                Json(serde_json::json!({ "session": { "id": "s1" } })),
                            )
                                .into_response()
                        } else if refused_by_hook {
                            (StatusCode::BAD_REQUEST, Json(flow(serde_json::json!([{ "id": 4000001, "type": "error", "text": "inactive" }])))).into_response()
                        } else {
                            (StatusCode::BAD_REQUEST, Json(flow(serde_json::json!([{ "id": 4000006, "type": "error", "text": "The provided credentials are invalid" }])))).into_response()
                        }
                    }
                }
            }),
        )
        .route(
            "/self-service/settings/flows",
            get(|| async {
                Json(serde_json::json!({
                    "id": "s1", "type": "browser",
                    "identity": { "traits": { "username": "somchai", "email": "somchai@arch.local" } },
                    "ui": { "action": "/self-service/settings?flow=s1", "method": "POST", "messages": [],
                            "nodes": [{ "type": "input", "group": "password", "attributes": { "name": "password", "type": "password" }, "messages": [] }] }
                }))
            }),
        )
        .route(
            "/self-service/settings",
            post({
                let state = state.clone();
                move || {
                    let state = state.clone();
                    async move {
                        state.settings_submits.fetch_add(1, Ordering::SeqCst);
                        Json(serde_json::json!({ "state": "success" }))
                    }
                }
            }),
        )
        .route("/sessions/whoami", get(|| async { (axum::response::AppendHeaders([(header::SET_COOKIE, "a=1"), (header::SET_COOKIE, "b=2")]), "hello").into_response() }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, state)
}

async fn gateway(s: &Security) -> (Router, FakeKratos) {
    let (url, kratos) = fake_kratos().await;
    let app = auth_gateway_routes_with_state(GatewayState { kratos_public_url: url, security: s.exec.clone() });
    (app, kratos)
}

fn json_login(identifier: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/self-service/login?flow=f1")
        .header("content-type", "application/json")
        .header("cookie", "csrf_token=abc")
        .body(Body::from(serde_json::json!({ "method": "password", "identifier": identifier, "password": password, "csrf_token": "abc" }).to_string()))
        .unwrap()
}

async fn message_ids(resp: axum::response::Response) -> (StatusCode, Vec<u64>, String) {
    let status = resp.status();
    let body: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap_or_default();
    let ids = body["ui"]["messages"].as_array().map(|m| m.iter().filter_map(|x| x["id"].as_u64()).collect()).unwrap_or_default();
    let text = body["ui"]["messages"][0]["text"].as_str().unwrap_or_default().to_string();
    (status, ids, text)
}

#[tokio::test]
async fn test_everything_but_the_password_submit_passes_through_untouched() {
    let (app, _) = gateway(&security()).await;
    let resp = app.clone().oneshot(Request::builder().uri("/sessions/whoami").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get_all(header::SET_COOKIE).iter().count(), 2, "every Set-Cookie survives");
    assert_eq!(text(resp).await, "hello");

    let flow_init = app.oneshot(Request::builder().uri("/self-service/login/browser").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(flow_init.headers().get(header::SET_COOKIE).unwrap(), "csrf_token=abc; Path=/");
}

#[tokio::test]
async fn test_a_right_password_signs_in_and_the_session_cookie_reaches_the_browser() {
    let (app, _) = gateway(&security()).await;
    let resp = app.oneshot(json_login("somchai", "right")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get(header::SET_COOKIE).unwrap().to_str().unwrap().starts_with("ory_kratos_session="));
}

#[tokio::test]
async fn test_five_wrong_passwords_lock_the_account_even_against_the_right_password() {
    let s = security();
    let (app, kratos) = gateway(&s).await;

    // Four wrong ones are plain "invalid credentials" answers
    for _ in 0..4 {
        let (status, ids, _) = message_ids(app.clone().oneshot(json_login("somchai", "wrong")).await.unwrap()).await;
        assert_eq!((status, ids), (StatusCode::BAD_REQUEST, vec![4000006]));
    }
    // The fifth locks: the page is told how long, instead of "invalid"
    let (status, ids, text) = message_ids(app.clone().oneshot(json_login("somchai", "wrong")).await.unwrap()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(ids, vec![4010001], "the lock message replaces the invalid-credentials one");
    assert!(text.contains("15") && text.contains("นาที") && text.contains("locked"), "{text}");
    assert_eq!(kratos.submits.load(Ordering::SeqCst), 5);

    // Locked: not even the right password is tried at Kratos
    let (status, ids, _) = message_ids(app.clone().oneshot(json_login("somchai", "right")).await.unwrap()).await;
    assert_eq!((status, ids), (StatusCode::BAD_REQUEST, vec![4010001]));
    assert_eq!(kratos.submits.load(Ordering::SeqCst), 5, "the locked attempt never reached Kratos");

    // Another spelling of the same account is locked too; another account is not
    let (_, ids, _) = message_ids(app.clone().oneshot(json_login("SomChai@arch.local", "right")).await.unwrap()).await;
    assert_eq!(ids, vec![4010001]);
    assert_eq!(app.oneshot(json_login("other", "right")).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn test_a_successful_sign_in_forgets_the_failures() {
    let (app, _) = gateway(&security()).await;
    for _ in 0..3 {
        app.clone().oneshot(json_login("somchai", "wrong")).await.unwrap();
    }
    assert_eq!(app.clone().oneshot(json_login("somchai", "right")).await.unwrap().status(), StatusCode::OK);
    for _ in 0..4 {
        let (_, ids, _) = message_ids(app.clone().oneshot(json_login("somchai", "wrong")).await.unwrap()).await;
        assert_eq!(ids, vec![4000006], "the count started again from zero");
    }
}

#[tokio::test]
async fn test_names_that_do_not_exist_lock_the_same_way() {
    let (app, kratos) = gateway(&security()).await;
    for _ in 0..5 {
        app.clone().oneshot(json_login("nobody", "wrong")).await.unwrap();
    }
    let (_, ids, _) = message_ids(app.oneshot(json_login("nobody", "wrong")).await.unwrap()).await;
    assert_eq!(ids, vec![4010001], "the answer does not tell whether the account exists");
    assert_eq!(kratos.submits.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn test_a_refusal_that_is_not_about_the_password_is_not_a_failed_attempt() {
    // The password was right but the login hook said no (inactive account, forced change): not brute force
    let (app, _) = gateway(&security()).await;
    for _ in 0..10 {
        let (_, ids, _) = message_ids(app.clone().oneshot(json_login("hookrefused", "right")).await.unwrap()).await;
        assert_eq!(ids, vec![4000001]);
    }
}

#[tokio::test]
async fn test_a_form_encoded_login_is_watched_like_a_json_one() {
    let (app, kratos) = gateway(&security()).await;
    let form = |password: &str| {
        Request::builder()
            .method("POST")
            .uri("/self-service/login?flow=f1")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(format!("method=password&identifier=somchai%40arch.local&password={password}&csrf_token=abc")))
            .unwrap()
    };
    for _ in 0..5 {
        app.clone().oneshot(form("wrong")).await.unwrap();
    }
    let (_, ids, _) = message_ids(app.oneshot(form("right")).await.unwrap()).await;
    assert_eq!(ids, vec![4010001]);
    assert_eq!(kratos.submits.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn test_a_submit_that_is_not_a_password_attempt_is_not_counted() {
    let (app, kratos) = gateway(&security()).await;
    let other = || {
        Request::builder()
            .method("POST")
            .uri("/self-service/login?flow=f1")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"method":"oidc","provider":"x"}"#))
            .unwrap()
    };
    for _ in 0..8 {
        app.clone().oneshot(other()).await.unwrap();
    }
    assert_eq!(kratos.submits.load(Ordering::SeqCst), 8, "all of them were forwarded, none locked");
}

fn settings_submit(password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/self-service/settings?flow=s1")
        .header("content-type", "application/json")
        .header("cookie", "ory_kratos_session=s")
        .body(Body::from(serde_json::json!({ "method": "password", "password": password, "csrf_token": "abc" }).to_string()))
        .unwrap()
}

#[tokio::test]
async fn test_a_new_password_that_breaks_the_policy_never_reaches_kratos() {
    let (app, kratos) = gateway(&security()).await;
    for weak in ["short1A", "alllowercase123", "NOLOWERCASE123", "NoDigitsHere", "xxSomchaixx1A"] {
        let resp = app.clone().oneshot(settings_submit(weak)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{weak}");
        let body: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
        // The flow comes back, with the rules on the password field and on the flow
        let field = &body["ui"]["nodes"][0]["messages"][0];
        assert_eq!(field["id"], 4010003, "{weak}");
        assert!(field["text"].as_str().unwrap().contains("policy"), "{weak}");
        assert_eq!(body["ui"]["messages"][0]["id"], 4010003);
    }
    assert_eq!(kratos.settings_submits.load(Ordering::SeqCst), 0, "none of them was sent to Kratos");
}

#[tokio::test]
async fn test_the_message_names_what_is_missing() {
    let (app, _) = gateway(&security()).await;
    let resp = app.oneshot(settings_submit("abc")).await.unwrap();
    let body: serde_json::Value = serde_json::from_str(&text(resp).await).unwrap();
    let message = body["ui"]["messages"][0]["text"].as_str().unwrap().to_string();
    assert!(message.contains("อย่างน้อย 8") && message.contains("uppercase") && message.contains("digit"), "{message}");
}

#[tokio::test]
async fn test_a_password_that_fits_the_policy_is_passed_on() {
    let (app, kratos) = gateway(&security()).await;
    let resp = app.oneshot(settings_submit("Good-Pass-2026")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(kratos.settings_submits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_a_settings_submit_that_is_not_a_password_is_not_checked() {
    // Changing the profile (name, e-mail) has no password in it
    let (app, kratos) = gateway(&security()).await;
    let req = Request::builder()
        .method("POST")
        .uri("/self-service/settings?flow=s1")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"method":"profile","traits.name.first":"x"}"#))
        .unwrap();
    assert_eq!(app.oneshot(req).await.unwrap().status(), StatusCode::OK);
    assert_eq!(kratos.settings_submits.load(Ordering::SeqCst), 1);
}
