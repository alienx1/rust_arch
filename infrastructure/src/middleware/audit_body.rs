//! Turns request / response headers and bodies into what the audit log stores: JSON text with
//! secrets masked and a size limit. Pure functions (unit-tested in `tests/audit_trail_test.rs`).

use axum::http::HeaderMap;
use crate::wire::hex_decode;
use serde_json::{Map, Number, Value};
use std::io::Cursor;

/// Bodies up to this size are buffered; larger ones are recorded as "<N bytes, not stored>"
pub const BODY_BUFFER_CAP: usize = 256 * 1024;
/// At most this many bytes of a body's JSON text are stored
pub const BODY_STORE_MAX: usize = 16 * 1024;
pub const MASK: &str = "***";

/// Header names that are always masked (besides any name containing token / secret / key)
const SECRET_HEADERS: [&str; 6] =
    ["cookie", "set-cookie", "authorization", "proxy-authorization", "x-api-key", "x-webhook-secret"];

/// `(?i)(password|passwd|secret|token|authorization|cookie|api[_-]?key|credential)` as a
/// substring test on a JSON object key
pub fn is_secret_key(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    ["password", "passwd", "secret", "token", "authorization", "cookie", "apikey", "api_key", "api-key", "credential"]
        .iter()
        .any(|needle| k.contains(needle))
}

fn is_secret_header(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    SECRET_HEADERS.contains(&n.as_str()) || n.contains("token") || n.contains("secret") || n.contains("key")
}

/// Header name -> value as a JSON object, secrets replaced by `***`
pub fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut out = Map::new();
    for (name, value) in headers {
        let text = if is_secret_header(name.as_str()) {
            MASK.to_string()
        } else {
            String::from_utf8_lossy(value.as_bytes()).into_owned()
        };
        match out.get_mut(name.as_str()) {
            Some(Value::String(existing)) => {
                existing.push_str(", ");
                existing.push_str(&text);
            }
            _ => {
                out.insert(name.as_str().to_string(), Value::String(text));
            }
        }
    }
    Value::Object(out)
}

/// Replaces the value of every object key that looks secret, at any depth
pub fn mask_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_secret_key(k) {
                    *v = Value::String(MASK.to_string());
                } else {
                    mask_json(v);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(mask_json),
        _ => {}
    }
}

/// CBOR -> JSON. Byte strings of 16 bytes are UUIDs (hyphenated); other byte strings become
/// `<binary N bytes>`; tags are dropped; non-text map keys are rendered through their JSON form.
pub fn cbor_to_json(value: &ciborium::Value) -> Value {
    use ciborium::Value as C;
    match value {
        C::Null => Value::Null,
        C::Bool(b) => Value::Bool(*b),
        C::Integer(i) => {
            let n = i128::from(*i);
            if let Ok(v) = i64::try_from(n) {
                Value::Number(v.into())
            } else if let Ok(v) = u64::try_from(n) {
                Value::Number(v.into())
            } else {
                Value::String(n.to_string())
            }
        }
        C::Float(f) => Number::from_f64(*f).map(Value::Number).unwrap_or(Value::Null),
        C::Text(t) => Value::String(t.clone()),
        C::Bytes(b) => match <[u8; 16]>::try_from(b.as_slice()) {
            Ok(arr) => Value::String(uuid::Uuid::from_bytes(arr).to_string()),
            Err(_) => Value::String(format!("<binary {} bytes>", b.len())),
        },
        C::Tag(_, inner) => cbor_to_json(inner),
        C::Array(items) => Value::Array(items.iter().map(cbor_to_json).collect()),
        C::Map(entries) => {
            let mut map = Map::new();
            for (k, v) in entries {
                let key = match cbor_to_json(k) {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                map.insert(key, cbor_to_json(v));
            }
            Value::Object(map)
        }
        _ => Value::Null,
    }
}

/// `<N bytes, not stored>`: body over the buffer cap (or of unknown size)
pub fn not_stored(len: usize) -> String {
    format!("<{len} bytes, not stored>")
}

/// Cuts `text` at `BODY_STORE_MAX` bytes (on a char boundary) and marks the cut
pub fn truncate_text(text: String) -> String {
    if text.len() <= BODY_STORE_MAX {
        return text;
    }
    let mut end = BODY_STORE_MAX;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…(truncated)", &text[..end])
}

fn media_type(content_type: Option<&str>) -> String {
    content_type
        .and_then(|c| c.split(';').next())
        .map(|c| c.trim().to_ascii_lowercase())
        .unwrap_or_default()
}

fn is_textual(media: &str) -> bool {
    media.is_empty()
        || media.starts_with("text/")
        || media.contains("json")
        || media.contains("xml")
        || media.contains("javascript")
        || media == "application/x-www-form-urlencoded"
}

/// Form bodies: `a=1&password=x` -> `a=1&password=***`
pub fn mask_form(text: &str) -> String {
    text.split('&')
        .map(|pair| match pair.split_once('=') {
            Some((k, _)) if is_secret_key(k) => format!("{k}={MASK}"),
            _ => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// The stored form of a body: `None` when empty. CBOR (`application/cbor`, `text/cbor-hex`) is
/// decoded and re-serialised as JSON, JSON is re-serialised compactly, forms and other text are
/// kept as (lossy) text, anything else is `<binary N bytes>`. Secrets in JSON / forms are masked
/// and the result is truncated to `BODY_STORE_MAX`.
pub fn body_to_text(content_type: Option<&str>, bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let media = media_type(content_type);
    let binary = || format!("<binary {} bytes>", bytes.len());

    let cbor_value = |raw: &[u8]| ciborium::from_reader::<ciborium::Value, _>(Cursor::new(raw)).ok();
    let json = if media == "application/cbor" {
        match cbor_value(bytes) {
            Some(v) => Some(cbor_to_json(&v)),
            None => return Some(binary()),
        }
    } else if media == "text/cbor-hex" {
        let decoded = std::str::from_utf8(bytes).ok().and_then(|t| hex_decode(t).ok());
        match decoded.as_deref().and_then(cbor_value) {
            Some(v) => Some(cbor_to_json(&v)),
            None => return Some(binary()),
        }
    } else if is_textual(&media) {
        let Ok(text) = std::str::from_utf8(bytes) else { return Some(binary()) };
        let trimmed = text.trim_start();
        let parsed = (media.contains("json") || trimmed.starts_with('{') || trimmed.starts_with('['))
            .then(|| serde_json::from_slice::<Value>(bytes).ok())
            .flatten();
        match parsed {
            Some(v) => Some(v),
            None if media == "application/x-www-form-urlencoded" => {
                return Some(truncate_text(mask_form(text)));
            }
            None => return Some(truncate_text(String::from_utf8_lossy(bytes).into_owned())),
        }
    } else {
        return Some(binary());
    };

    let mut json = json?;
    mask_json(&mut json);
    Some(truncate_text(json.to_string()))
}
