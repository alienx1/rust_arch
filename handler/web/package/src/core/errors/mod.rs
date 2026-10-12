//! Core error definitions for WebAssembly.

pub mod auth;
pub mod http;

pub use auth::AuthError;
pub use http::HttpError;
