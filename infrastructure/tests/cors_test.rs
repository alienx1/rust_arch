//! Tests for Infrastructure CORS Middleware.
//!
//! Verifies preflight handling, allowed origins loaded from .env,
//! credentials support for Kratos sessions, and rejection of untrusted origins.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    routing::get,
};
use infrastructure::middleware::cors::{CorsConfig, cors_layer, cors_layer_from_env};
use tower::ServiceExt;

#[tokio::test]
async fn test_cors_from_env_loads_dotenv_and_allows_origins() {
    // Tests that CorsConfig::from_env() reads from .env and allows configured origin
    let config = CorsConfig::from_env();
    assert!(
        !config.allowed_origins.is_empty(),
        "Origins should be loaded from .env"
    );
    assert!(config.allow_credentials, "Credentials should be enabled");

    let app = Router::new()
        .route("/api/v1/test", get(|| async { "ok" }))
        .layer(cors_layer_from_env());

    let test_origin = &config.allowed_origins[0];

    let req = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/v1/test")
        .header(header::ORIGIN, test_origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .header(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            "authorization, content-type, cookie",
        )
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        test_origin
    );
    assert_eq!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .unwrap(),
        "true"
    );
}

#[tokio::test]
async fn test_cors_rejects_unlisted_origin() {
    let config = CorsConfig::from_env();
    let app = Router::new()
        .route("/api/v1/test", get(|| async { "ok" }))
        .layer(cors_layer(&config));

    let req = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/v1/test")
        .header(header::ORIGIN, "http://unauthorized-evil-domain.com")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Tower-http CORS will not include Access-Control-Allow-Origin for unauthorized origins
    assert!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}

#[tokio::test]
async fn test_cors_simple_get_request() {
    let config = CorsConfig::from_env();
    let test_origin = &config.allowed_origins[0];

    let app = Router::new()
        .route("/api/v1/test", get(|| async { "hello cors" }))
        .layer(cors_layer(&config));

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/test")
        .header(header::ORIGIN, test_origin)
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        test_origin
    );
    assert_eq!(
        res.headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .unwrap(),
        "true"
    );
}
