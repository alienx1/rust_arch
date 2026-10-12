//! HTTP Client / Requestor module for Leptos WebAssembly applications.

pub mod builder;
pub mod client;
pub mod response;

pub use crate::core::errors::HttpError;
pub use builder::{HttpMethod, RequestBuilder};
pub use client::HttpClient;
pub use response::HttpResponse;

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
