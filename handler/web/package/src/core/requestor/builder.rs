use super::response::HttpResponse;
use crate::core::errors::HttpError;
use gloo_net::http::Request;
use serde::Serialize;
use std::collections::HashMap;
use web_sys::RequestCredentials;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Clone, Debug)]
pub enum RequestPayload {
    Text(String),
    Binary(Vec<u8>),
}

pub struct RequestBuilder {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    query: Vec<(String, String)>,
    payload: Option<RequestPayload>,
    credentials: RequestCredentials,
}

impl RequestBuilder {
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        // Request CBOR by default for high-performance Rust-WASM transport, falling back to JSON
        headers.insert(
            "Accept".to_string(),
            "application/cbor, application/json;q=0.9, */*;q=0.8".to_string(),
        );

        // The API localizes its error messages by language
        headers.insert("Accept-Language".to_string(), crate::core::i18n::get_saved_language());

        // The company chosen in the topbar switcher (the server checks it)
        if let Some(company) = crate::core::company_scope::get_selected_company() {
            headers.insert("X-Company-Id".to_string(), company);
        }
        if let Some(branch) = crate::core::company_scope::get_selected_branch() {
            headers.insert("X-Branch-Id".to_string(), branch);
        }

        Self {
            url: url.into(),
            method,
            headers,
            query: Vec::new(),
            payload: None,
            credentials: RequestCredentials::Include, // Default to Include cookies for Kratos sessions
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((key.into(), value.into()));
        self
    }

    /// Sets request body as JSON (`Content-Type: application/json`).
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, HttpError> {
        let json_str =
            serde_json::to_string(data).map_err(|e| HttpError::Serialization(e.to_string()))?;
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.payload = Some(RequestPayload::Text(json_str));
        Ok(self)
    }

    /// Sets request body as binary CBOR (`Content-Type: application/cbor`).
    pub fn cbor<T: Serialize>(mut self, data: &T) -> Result<Self, HttpError> {
        let buf = encode_cbor(data)?;
        self.headers
            .insert("Content-Type".to_string(), "application/cbor".to_string());
        self.payload = Some(RequestPayload::Binary(buf));
        Ok(self)
    }

    /// Sets request body as hex-encoded CBOR (`Content-Type: text/cbor-hex`) and asks for
    /// hex responses too, for readable payloads while debugging.
    pub fn cbor_hex<T: Serialize>(mut self, data: &T) -> Result<Self, HttpError> {
        let hex_str = super::hex_encode(&encode_cbor(data)?);
        self.headers
            .insert("Content-Type".to_string(), "text/cbor-hex".to_string());
        self.headers.insert(
            "Accept".to_string(),
            "text/cbor-hex, application/json;q=0.9, */*;q=0.8".to_string(),
        );
        self.payload = Some(RequestPayload::Text(hex_str));
        Ok(self)
    }

    /// Sets the request body to raw bytes of the given type (e.g. an image file).
    pub fn bytes(mut self, data: Vec<u8>, content_type: &str) -> Self {
        self.headers.insert("Content-Type".to_string(), content_type.to_string());
        self.payload = Some(RequestPayload::Binary(data));
        self
    }

    pub fn credentials(mut self, creds: RequestCredentials) -> Self {
        self.credentials = creds;
        self
    }

    pub async fn send(self) -> Result<HttpResponse, HttpError> {
        let mut full_url = self.url;
        if !self.query.is_empty() {
            let mut first = !full_url.contains('?');
            for (k, v) in self.query {
                let sep = if first { '?' } else { '&' };
                full_url.push(sep);
                full_url.push_str(&url_encode(&k));
                full_url.push('=');
                full_url.push_str(&url_encode(&v));
                first = false;
            }
        }

        let mut req = match self.method {
            HttpMethod::Get => Request::get(&full_url),
            HttpMethod::Post => Request::post(&full_url),
            HttpMethod::Put => Request::put(&full_url),
            HttpMethod::Delete => Request::delete(&full_url),
            HttpMethod::Patch => Request::patch(&full_url),
        };

        req = req.credentials(self.credentials);

        for (k, v) in self.headers {
            req = req.header(&k, &v);
        }

        let gloo_resp = match self.payload {
            Some(RequestPayload::Text(ref text)) => {
                req.body(text.as_str())
                    .map_err(|e| HttpError::Network(e.to_string()))?
                    .send()
                    .await
                    .map_err(|e| HttpError::Network(e.to_string()))?
            }
            Some(RequestPayload::Binary(ref bytes)) => {
                let uint8_array = js_sys::Uint8Array::from(bytes.as_slice());
                req.body(uint8_array)
                    .map_err(|e| HttpError::Network(e.to_string()))?
                    .send()
                    .await
                    .map_err(|e| HttpError::Network(e.to_string()))?
            }
            None => {
                req.send()
                    .await
                    .map_err(|e| HttpError::Network(e.to_string()))?
            }
        };

        HttpResponse::from_gloo(gloo_resp).await
    }
}

fn encode_cbor<T: Serialize>(data: &T) -> Result<Vec<u8>, HttpError> {
    let mut buf = Vec::new();
    ciborium::into_writer(data, &mut buf)
        .map_err(|e| HttpError::Serialization(format!("CBOR encode error: {e}")))?;
    Ok(buf)
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
