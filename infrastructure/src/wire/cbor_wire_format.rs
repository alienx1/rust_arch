//! CBOR Wire Format & Content Negotiation for Axum
//!
//! Provides high-performance binary serialization using CBOR (RFC 8949)
//! via `ciborium`, along with content negotiation between three wire formats:
//! - `application/cbor`: raw binary CBOR (default for the Web/WASM client).
//! - `text/cbor-hex`: the same CBOR as lowercase hex text (curl, logs, debugging).
//! - `application/json`: fallback for Swagger and standard clients.

use axum::{
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Request},
    http::{
        HeaderMap, StatusCode, header,
        request::Parts,
    },
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use std::io::Cursor;

/// Content-Type for raw binary Concise Binary Object Representation.
pub const APPLICATION_CBOR: &str = "application/cbor";

/// Media type of hex-encoded CBOR bodies (as sent in `Accept` / `Content-Type`).
pub const CBOR_HEX: &str = "text/cbor-hex";

/// Content-Type of hex-encoded CBOR response bodies.
///
/// A `text/*` type on purpose: the body is lowercase hex text, and `application/cbor` makes
/// devtools and proxies treat it as binary and re-wrap it in base64.
pub const CBOR_HEX_CONTENT_TYPE: &str = "text/cbor-hex; charset=utf-8";

/// Axum wrapper for binary CBOR response serialization and request extraction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cbor<T>(pub T);

impl<T> Cbor<T> {
    pub fn new(val: T) -> Self {
        Self(val)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Serialize> IntoResponse for Cbor<T> {
    fn into_response(self) -> Response {
        match to_cbor_vec(&self.0) {
            Ok(buf) => (StatusCode::OK, [(header::CONTENT_TYPE, APPLICATION_CBOR)], buf).into_response(),
            Err(err) => cbor_serialization_error(err),
        }
    }
}

impl<T, S> FromRequest<S> for Cbor<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read request body: {e}")))?;

        from_cbor_slice(bytes.as_ref())
            .map(Cbor)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid CBOR payload: {e}")))
    }
}

/// Axum wrapper for hex-encoded CBOR responses (`text/cbor-hex`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CborHex<T>(pub T);

impl<T: Serialize> IntoResponse for CborHex<T> {
    fn into_response(self) -> Response {
        match to_cbor_hex(&self.0) {
            Ok(hex_str) => {
                (StatusCode::OK, [(header::CONTENT_TYPE, CBOR_HEX_CONTENT_TYPE)], hex_str).into_response()
            }
            Err(err) => cbor_serialization_error(err),
        }
    }
}

fn cbor_serialization_error(err: ciborium::ser::Error<std::io::Error>) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("CBOR serialization error: {err}"),
    )
        .into_response()
}

/// Request-body extractor negotiated by `Content-Type`.
///
/// - `application/cbor`: raw binary CBOR.
/// - `text/cbor-hex`: hex-encoded CBOR.
/// - anything else: JSON, for curl, Swagger, and standard clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireBody<T>(pub T);

impl<T, S> FromRequest<S> for WireBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let format = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map_or(WireFormat::Json, WireFormat::from_media_type);
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read request body: {e}")))?;

        let invalid_cbor = |e: String| (StatusCode::BAD_REQUEST, format!("Invalid CBOR payload: {e}"));
        match format {
            WireFormat::Cbor => ciborium::from_reader(Cursor::new(bytes.as_ref()))
                .map(WireBody)
                .map_err(|e| invalid_cbor(e.to_string())),
            WireFormat::CborHex => {
                let text = std::str::from_utf8(&bytes).map_err(|e| invalid_cbor(e.to_string()))?;
                let decoded = hex_decode(text).map_err(invalid_cbor)?;
                ciborium::from_reader(Cursor::new(decoded))
                    .map(WireBody)
                    .map_err(|e| invalid_cbor(e.to_string()))
            }
            WireFormat::Json => serde_json::from_slice(bytes.as_ref())
                .map(WireBody)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON payload: {e}"))),
        }
    }
}

/// Content-Negotiation format extractor for Axum handlers.
///
/// Inspects the incoming `Accept` header:
/// - `text/cbor-hex` selects `WireFormat::CborHex`.
/// - `application/cbor` selects `WireFormat::Cbor` (binary).
/// - Otherwise defaults to `WireFormat::Json` for curl, Swagger, and standard clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireFormat {
    Json,
    Cbor,
    CborHex,
}

impl WireFormat {
    /// Detect the response wire format from the `Accept` header.
    pub fn from_headers(headers: &HeaderMap) -> Self {
        headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map_or(WireFormat::Json, Self::from_media_type)
    }

    /// Map an `Accept` / `Content-Type` value to a wire format (hex wins when both are listed).
    pub fn from_media_type(value: &str) -> Self {
        if value.contains(CBOR_HEX) {
            WireFormat::CborHex
        } else if value.contains(APPLICATION_CBOR) {
            WireFormat::Cbor
        } else {
            WireFormat::Json
        }
    }

    /// Wrap serializable data into an HTTP response using the negotiated format.
    pub fn respond<T: Serialize>(self, data: T) -> Response {
        match self {
            WireFormat::Cbor => Cbor(data).into_response(),
            WireFormat::CborHex => CborHex(data).into_response(),
            WireFormat::Json => axum::Json(data).into_response(),
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for WireFormat {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_headers(&parts.headers))
    }
}

/// Serializes any value into a CBOR byte vector.
pub fn to_cbor_vec<T: Serialize>(val: &T) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
    let mut buf = Vec::new();
    ciborium::into_writer(val, &mut buf)?;
    Ok(buf)
}

/// Serializes any value into a hex-encoded CBOR string.
pub fn to_cbor_hex<T: Serialize>(val: &T) -> Result<String, ciborium::ser::Error<std::io::Error>> {
    let buf = to_cbor_vec(val)?;
    Ok(hex_encode(&buf))
}

/// Deserializes a CBOR slice (either hex-encoded ASCII or raw binary) into any target type.
pub fn from_cbor_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ciborium::de::Error<std::io::Error>> {
    if let Ok(text) = std::str::from_utf8(bytes) {
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.len() % 2 == 0 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(decoded) = hex_decode(trimmed) {
                return ciborium::from_reader(Cursor::new(decoded));
            }
        }
    }
    ciborium::from_reader(Cursor::new(bytes))
}

/// Encodes bytes into a lowercase hexadecimal string.
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

/// Decodes a hexadecimal string into bytes.
pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("Hex string must have an even length".to_string());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars = s.as_bytes();
    for i in (0..chars.len()).step_by(2) {
        let hi = hex_val(chars[i]).ok_or_else(|| format!("Invalid hex char '{}'", chars[i] as char))?;
        let lo = hex_val(chars[i + 1]).ok_or_else(|| format!("Invalid hex char '{}'", chars[i + 1] as char))?;
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Axum middleware that re-encodes error responses (status >= 400) as CBOR when the client
/// negotiated CBOR: the `Accept` header decides, falling back to the request `Content-Type`.
/// `application/cbor` yields binary CBOR, `text/cbor-hex` yields hex CBOR.
///
/// JSON error bodies keep their shape; plain-text rejections (e.g. invalid payload) are wrapped
/// into the standard `{ success, code, message }` error body first.
pub async fn cbor_error_responses(req: Request, next: axum::middleware::Next) -> Response {
    let format = match WireFormat::from_headers(req.headers()) {
        WireFormat::Json => req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map_or(WireFormat::Json, WireFormat::from_media_type),
        negotiated => negotiated,
    };

    let resp = next.run(req).await;
    let status = resp.status();
    if format == WireFormat::Json || !(status.is_client_error() || status.is_server_error()) {
        return resp;
    }

    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    if content_type.contains("cbor") {
        return resp;
    }

    let (mut parts, body) = resp.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, 1024 * 1024).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed to read error body").into_response();
    };

    let value: serde_json::Value = if content_type.contains("json") {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| plain_error_body(status, &bytes))
    } else {
        plain_error_body(status, &bytes)
    };

    let Ok(buf) = to_cbor_vec(&value) else {
        return Response::from_parts(parts, axum::body::Body::from(bytes));
    };
    let (content_type, body) = if format == WireFormat::CborHex {
        (CBOR_HEX_CONTENT_TYPE, axum::body::Body::from(hex_encode(&buf)))
    } else {
        (APPLICATION_CBOR, axum::body::Body::from(buf))
    };
    parts.headers.remove(header::CONTENT_LENGTH);
    parts
        .headers
        .insert(header::CONTENT_TYPE, header::HeaderValue::from_static(content_type));
    Response::from_parts(parts, body)
}

fn plain_error_body(status: StatusCode, bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "success": false,
        "code": status.canonical_reason().unwrap_or("ERROR").to_uppercase().replace(' ', "_"),
        "message": String::from_utf8_lossy(bytes),
    })
}
