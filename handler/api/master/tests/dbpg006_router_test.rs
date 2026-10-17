//! Integration tests for DBPG006 router endpoints (`/db/dbpg006/...`): authorization, paging
//! parameters, entry identification by query and error mapping.
//! Located at `handler/api/master/tests/dbpg006_router_test.rs`.

use api_master::routers::{DBPG006_PATH, Dbpg006State, dbpg006_routes_with_state};
use application_master::features::db::dbpg006::{
    CreateLabelCommand, Dbpg006Commands, Dbpg006Error, LabelCategoryDto, LabelEntryDto,
    LabelFilterQuery, LabelPageDto, LabelValueDto, LanguageOptionDto, UpdateLabelCommand,
};
use application_master::features::su::access_control::{
    AccessControl, AccessError, Actor, PermissionAction,
};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use infrastructure::middleware::AuthenticatedUser;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// `admin` reads, writes and approves; `writer` reads and writes; `reader` only reads; anyone else has nothing.
struct MockAccess;

#[async_trait]
impl AccessControl for MockAccess {
    async fn authorize(
        &self,
        username: &str,
        program_code: &str,
        action: PermissionAction,
    ) -> Result<Option<Actor>, AccessError> {
        assert_eq!(program_code, "DBPG006");
        let allowed = match (username, action) {
            ("admin", _) => true,
            ("writer", PermissionAction::Read | PermissionAction::Write) => true,
            ("reader", PermissionAction::Read) => true,
            _ => false,
        };
        Ok(allowed.then(|| Actor { user_id: Uuid::nil(), username: username.to_string() }))
    }
}

struct MockCommands;

fn entry(category: &str, key: &str) -> LabelEntryDto {
    LabelEntryDto {
        category: category.to_string(),
        key: key.to_string(),
        values: vec![LabelValueDto {
            language_id: Uuid::now_v7(),
            language_code: "th".to_string(),
            value: "ข้อความ".to_string(),
        }],
        row_version: 1,
    }
}

#[async_trait]
impl Dbpg006Commands for MockCommands {
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        Ok(vec![LanguageOptionDto {
            id: Uuid::now_v7(),
            code: "th".to_string(),
            name_th: "ไทย".to_string(),
            name_en: "Thai".to_string(),
        }])
    }
    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        Ok(vec![LabelCategoryDto { category: "menu".to_string(), entries: 2 }])
    }
    async fn list_labels(&self, f: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error> {
        Ok(LabelPageDto {
            // Echo the category / search so the test can see they were parsed
            items: vec![entry(
                f.category.as_deref().unwrap_or("all"),
                f.search.as_deref().unwrap_or("none"),
            )],
            total: 3,
            page: f.page.unwrap_or(1),
            limit: f.limit.unwrap_or(25),
        })
    }
    async fn get_label(&self, category: &str, key: &str) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        Ok((key != "missing").then(|| entry(category, key)))
    }
    async fn create_label(&self, cmd: CreateLabelCommand, actor: Option<Uuid>) -> Result<LabelEntryDto, Dbpg006Error> {
        assert_eq!(actor, Some(Uuid::nil()), "the authorized user is recorded as the actor");
        match cmd.key.as_str() {
            "dup" => Err(Dbpg006Error::Duplicate("dup".to_string())),
            "" => Err(Dbpg006Error::Validation("Key is required".to_string())),
            _ => Ok(entry(&cmd.category, &cmd.key)),
        }
    }
    async fn update_label(&self, cmd: UpdateLabelCommand, _actor: Option<Uuid>) -> Result<LabelEntryDto, Dbpg006Error> {
        match cmd.key.as_str() {
            "missing" => Err(Dbpg006Error::NotFound("missing".to_string())),
            _ => Ok(entry(&cmd.category, &cmd.key)),
        }
    }
    async fn delete_label(&self, _category: &str, key: &str) -> Result<(), Dbpg006Error> {
        if key == "missing" {
            return Err(Dbpg006Error::NotFound("missing".to_string()));
        }
        Ok(())
    }
}

fn user(username: &str) -> AuthenticatedUser {
    AuthenticatedUser {
        identity_id: "i".to_string(),
        session_id: "s".to_string(),
        email: Some(format!("{username}@arch.local")),
        traits: serde_json::json!({}),
    }
}

fn router(username: &str) -> Router {
    let state = Dbpg006State { commands: Arc::new(MockCommands), access: Arc::new(MockAccess) };
    Router::new()
        .nest(DBPG006_PATH, dbpg006_routes_with_state(state))
        .layer(Extension(user(username)))
}

fn request(method: &str, uri: &str) -> Request<Body> {
    Request::builder().uri(uri).method(method).body(Body::empty()).unwrap()
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(method)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn label_body(key: &str) -> serde_json::Value {
    serde_json::json!({
        "category": "menu", "key": key,
        "values": [{ "language_id": Uuid::now_v7(), "value": "ข้อความ" }],
        "row_version": 1
    })
}

async fn status_and_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null))
}

#[tokio::test]
async fn test_list_parses_filter_and_paging_and_needs_read() {
    let (status, body) = status_and_json(
        router("reader")
            .oneshot(request("GET", "/db/dbpg006/labels?category=menu&search=a%20b&page=2&limit=10"))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"][0]["category"], "menu");
    assert_eq!(body["items"][0]["key"], "a b", "the search term is percent-decoded");
    assert_eq!((body["page"].as_u64(), body["limit"].as_u64(), body["total"].as_u64()), (Some(2), Some(10), Some(3)));

    for uri in [
        "/db/dbpg006/labels",
        "/db/dbpg006/languages",
        "/db/dbpg006/categories",
        "/db/dbpg006/labels/entry?category=menu&key=k",
    ] {
        let (status, body) = status_and_json(router("stranger").oneshot(request("GET", uri)).await.unwrap()).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}");
        assert_eq!(body["code"], "FORBIDDEN");
    }
}

#[tokio::test]
async fn test_languages_and_categories_are_readable() {
    let (status, body) = status_and_json(router("reader").oneshot(request("GET", "/db/dbpg006/languages")).await.unwrap()).await;
    assert_eq!((status, body[0]["code"].as_str()), (StatusCode::OK, Some("th")));
    let (status, body) = status_and_json(router("reader").oneshot(request("GET", "/db/dbpg006/categories")).await.unwrap()).await;
    assert_eq!((status, body[0]["entries"].as_u64()), (StatusCode::OK, Some(2)));
}

#[tokio::test]
async fn test_get_entry_by_query_and_404() {
    let (status, body) = status_and_json(
        router("reader")
            .oneshot(request("GET", "/db/dbpg006/labels/entry?category=menu&key=column.thai%20name"))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["key"], "column.thai name");

    let missing = router("reader")
        .oneshot(request("GET", "/db/dbpg006/labels/entry?category=menu&key=missing"))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_writes_need_write() {
    let create = || json_request("POST", "/db/dbpg006/labels", label_body("new"));

    assert_eq!(router("reader").oneshot(create()).await.unwrap().status(), StatusCode::FORBIDDEN);
    let (status, body) = status_and_json(router("writer").oneshot(create()).await.unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["key"], "new");

    let put = || json_request("PUT", "/db/dbpg006/labels", label_body("k"));
    assert_eq!(router("reader").oneshot(put()).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("writer").oneshot(put()).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn test_error_status_codes() {
    let app = router("writer");
    let (status, body) = status_and_json(
        app.clone().oneshot(json_request("POST", "/db/dbpg006/labels", label_body("dup"))).await.unwrap(),
    )
    .await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::CONFLICT, Some("DUPLICATE")));

    let (status, body) = status_and_json(
        app.clone().oneshot(json_request("POST", "/db/dbpg006/labels", label_body(""))).await.unwrap(),
    )
    .await;
    assert_eq!((status, body["code"].as_str()), (StatusCode::BAD_REQUEST, Some("VALIDATION_ERROR")));

    let missing = app
        .oneshot(json_request("PUT", "/db/dbpg006/labels", label_body("missing")))
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_needs_approve_and_maps_not_found() {
    let delete = |key: &str| request("DELETE", &format!("/db/dbpg006/labels?category=menu&key={key}"));

    assert_eq!(router("reader").oneshot(delete("k")).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("writer").oneshot(delete("k")).await.unwrap().status(), StatusCode::FORBIDDEN);
    assert_eq!(router("admin").oneshot(delete("k")).await.unwrap().status(), StatusCode::NO_CONTENT);
    assert_eq!(router("admin").oneshot(delete("missing")).await.unwrap().status(), StatusCode::NOT_FOUND);
}
