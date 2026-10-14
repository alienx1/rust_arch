//! Error localization middleware.
//!
//! Error bodies are produced in English as `{ success: false, code, message }` (JSON, CBOR or
//! CBOR-hex, see `infrastructure::wire`). For responses with status >= 400 this middleware looks
//! up the label `error.<code>` in the language of the `Accept-Language` header and replaces
//! `message` with that template, substituting `{detail}` with the original English message.
//! Without a template (or when the lookup fails) the response is left untouched.

use application_core::traits::ErrorTemplateSource;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::wire::{
    APPLICATION_CBOR, CBOR_HEX_CONTENT_TYPE, from_cbor_slice, hex_encode, to_cbor_vec,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Languages with seeded templates; anything else falls back to `DEFAULT_LANGUAGE`.
const KNOWN_LANGUAGES: [&str; 2] = ["th", "en"];
const DEFAULT_LANGUAGE: &str = "th";
/// How long a language's templates are served from memory before they are reloaded.
const CACHE_TTL: Duration = Duration::from_secs(30);
/// Upper bound of an error body that is inspected.
const MAX_BODY_BYTES: usize = 1024 * 1024;

type Templates = Arc<HashMap<String, String>>;

/// Resolves error templates per language with a short-lived in-memory cache.
pub struct ErrorLocalizer {
    source: Arc<dyn ErrorTemplateSource>,
    cache: Mutex<HashMap<String, (Instant, Templates)>>,
}

impl ErrorLocalizer {
    pub fn new(source: Arc<dyn ErrorTemplateSource>) -> Self {
        Self {
            source,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Templates of `language`, from cache while fresh. On a failed reload the stale copy (if
    /// any) keeps being served and is retried after the next TTL; `None` when nothing is known.
    async fn templates(&self, language: &str) -> Option<Templates> {
        let stale = {
            let cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
            match cache.get(language) {
                Some((loaded, templates)) if loaded.elapsed() < CACHE_TTL => {
                    return Some(templates.clone());
                }
                Some((_, templates)) => Some(templates.clone()),
                None => None,
            }
        };

        let loaded = match self.source.error_templates(language).await {
            Ok(map) => Some(Arc::new(map)),
            Err(err) => {
                tracing::warn!("Could not load error templates for '{language}': {err}");
                stale
            }
        };
        if let Some(templates) = &loaded {
            self.cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(language.to_string(), (Instant::now(), templates.clone()));
        }
        loaded
    }
}

/// Primary subtag of the first `Accept-Language` tag (`th-TH,en;q=0.8` -> `th`), restricted to
/// the known languages and defaulting to Thai.
fn requested_language(headers: &HeaderMap) -> &'static str {
    let primary = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|tag| tag.split(';').next())
        .and_then(|tag| tag.trim().split('-').next())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    KNOWN_LANGUAGES
        .into_iter()
        .find(|l| *l == primary)
        .unwrap_or(DEFAULT_LANGUAGE)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyEncoding {
    Json,
    Cbor,
    CborHex,
}

impl BodyEncoding {
    fn from_content_type(content_type: &str) -> Option<Self> {
        if content_type.contains("cbor-hex") {
            Some(Self::CborHex)
        } else if content_type.contains("cbor") {
            Some(Self::Cbor)
        } else if content_type.contains("json") {
            Some(Self::Json)
        } else {
            None
        }
    }

    fn decode(self, bytes: &[u8]) -> Option<serde_json::Value> {
        match self {
            Self::Json => serde_json::from_slice(bytes).ok(),
            Self::Cbor | Self::CborHex => from_cbor_slice(bytes).ok(),
        }
    }

    /// Encoded body plus the matching `Content-Type`.
    fn encode(self, value: &serde_json::Value) -> Option<(Vec<u8>, HeaderValue)> {
        match self {
            Self::Json => serde_json::to_vec(value)
                .ok()
                .map(|b| (b, HeaderValue::from_static("application/json"))),
            Self::Cbor => to_cbor_vec(value)
                .ok()
                .map(|b| (b, HeaderValue::from_static(APPLICATION_CBOR))),
            Self::CborHex => to_cbor_vec(value)
                .ok()
                .map(|b| (hex_encode(&b).into_bytes(), HeaderValue::from_static(CBOR_HEX_CONTENT_TYPE))),
        }
    }
}

/// Axum middleware: localizes the `message` of error bodies (see module docs).
pub async fn localize_error_responses(
    State(localizer): State<Arc<ErrorLocalizer>>,
    req: Request,
    next: Next,
) -> Response {
    let language = requested_language(req.headers());
    let resp = next.run(req).await;
    if resp.status().as_u16() < 400 {
        return resp;
    }

    let encoding = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(BodyEncoding::from_content_type);
    let Some(encoding) = encoding else {
        return resp;
    };

    let (mut parts, body) = resp.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, MAX_BODY_BYTES).await else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "failed to read error body").into_response();
    };
    let untouched = |parts, bytes| Response::from_parts(parts, Body::from(bytes));

    let Some(mut value) = encoding.decode(&bytes) else {
        return untouched(parts, bytes);
    };
    let (Some(code), Some(original)) = (
        value.get("code").and_then(|v| v.as_str()).map(str::to_string),
        value.get("message").and_then(|v| v.as_str()).map(str::to_string),
    ) else {
        return untouched(parts, bytes);
    };

    let Some(templates) = localizer.templates(language).await else {
        return untouched(parts, bytes);
    };
    let Some(template) = templates.get(&code) else {
        return untouched(parts, bytes);
    };
    value["message"] = serde_json::Value::String(template.replace("{detail}", &original));

    let Some((encoded, content_type)) = encoding.encode(&value) else {
        return untouched(parts, bytes);
    };
    parts.headers.insert(header::CONTENT_TYPE, content_type);
    parts.headers.insert(header::CONTENT_LENGTH, HeaderValue::from(encoded.len()));
    Response::from_parts(parts, Body::from(encoded))
}
