//! Axum routers for api-report.

pub mod base_router;
pub mod rppg000_router;

pub use base_router::build_app_router;
pub use rppg000_router::{RPPG000_PATH, Rppg000ApiError, rppg000_routes};
