//! Request language middleware: runs the request with the `Accept-Language` language in scope,
//! so queries can return names in that language (see `application_core::request_language`).

use application_core::request_language::{Language, scope};
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

pub async fn set_request_language(req: Request, next: Next) -> Response {
    let language = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map(Language::from_accept_language)
        .unwrap_or_default();
    scope(language, next.run(req)).await
}
