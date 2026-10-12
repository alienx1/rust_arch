//! Axum routers for api-notification.

pub mod base_router;
pub mod ntpg000_router;

pub use base_router::build_app_router;
pub use ntpg000_router::{Ntpg000ApiError, ntpg000_routes};
