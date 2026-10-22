//! Master API Service Library (api-master)
//!
//! Provides service connections, base routers, and domain feature routers (e.g. localization/i18n).

pub mod cache_policy;
pub mod connection;
pub mod routers;
pub mod services;
pub mod wiring;
pub mod view_scope_layer;

pub use connection::*;
pub use routers::*;
