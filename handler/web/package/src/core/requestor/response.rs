use crate::core::errors::HttpError;
use gloo_net::http::Response as GlooResponse;
use serde::de::DeserializeOwned;
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct HttpResponse {
    status: u16,
    ok: bool,
    body_bytes: Vec<u8>,
    content_type: String,
}

impl HttpResponse {
    pub async fn from_gloo(resp: GlooResponse) -> Result<Self, HttpError> {
        let status = resp.status();
        let ok = resp.ok();
        let content_type = resp.headers().get("content-type").unwrap_or_default();
        let body_bytes = resp
            .binary()
            .await
            .map_err(|e| HttpError::Network(e.to_string()))?;

        if !ok {
            let body_text = error_message(&content_type, &body_bytes);
            if status == 401 {
                return Err(HttpError::Unauthorized);
            } else if status == 403 {
                return Err(HttpError::Forbidden);
            } else if status == 404 {
                return Err(HttpError::NotFound(body_text));
            } else {
                return Err(HttpError::ApiError {
                    status,
                    message: body_text,
                });
            }
        }

        Ok(Self {
            status,
            ok,
            body_bytes,
            content_type,
        })
    }

    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn is_success(&self) -> bool {
        self.ok
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn bytes(&self) -> &[u8] {
        &self.body_bytes
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body_bytes).to_string()
    }

    /// Deserializes response body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_slice(&self.body_bytes).map_err(|e| HttpError::Serialization(e.to_string()))
    }

    /// Deserializes response body as CBOR: hex text for `text/cbor-hex`, raw binary otherwise.
    pub fn cbor<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        let decode_err = |e: String| HttpError::Serialization(format!("CBOR decode error: {e}"));
        if self.content_type.contains("cbor-hex") {
            let text = std::str::from_utf8(&self.body_bytes).map_err(|e| decode_err(e.to_string()))?;
            let bytes = super::hex_decode(text.trim())
                .map_err(|e| HttpError::Serialization(format!("CBOR hex decode error: {e}")))?;
            return ciborium::from_reader(Cursor::new(bytes)).map_err(|e| decode_err(e.to_string()));
        }
        ciborium::from_reader(Cursor::new(&self.body_bytes)).map_err(|e| decode_err(e.to_string()))
    }

    /// Automatically deserializes based on the `Content-Type` response header.
    /// If a CBOR type (`text/cbor-hex` or `application/cbor`) is returned, decodes via CBOR.
    /// Otherwise decodes via JSON.
    pub fn auto<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        if self.content_type.contains("cbor") {
            self.cbor()
        } else {
            self.json()
        }
    }
}

/// Human-readable text of an error body: the `message` of the API's `{code, message, success}`
/// error object (JSON, CBOR or CBOR hex), or the raw text when the body has another shape.
fn error_message(content_type: &str, bytes: &[u8]) -> String {
    let from_json = |b: &[u8]| {
        serde_json::from_slice::<serde_json::Value>(b)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str().map(str::to_string)))
    };
    let from_cbor = |b: &[u8]| {
        let value: ciborium::value::Value = ciborium::from_reader(Cursor::new(b)).ok()?;
        value.into_map().ok()?.into_iter().find_map(|(k, v)| match (k, v) {
            (ciborium::value::Value::Text(k), ciborium::value::Value::Text(v)) if k == "message" => Some(v),
            _ => None,
        })
    };
    let decoded = if content_type.contains("cbor-hex") {
        std::str::from_utf8(bytes)
            .ok()
            .and_then(|t| super::hex_decode(t.trim()).ok())
            .and_then(|b| from_cbor(&b))
    } else if content_type.contains("cbor") {
        from_cbor(bytes)
    } else {
        from_json(bytes)
    };
    decoded.unwrap_or_else(|| String::from_utf8_lossy(bytes).to_string())
}
