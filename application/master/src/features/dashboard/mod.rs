//! # Dashboard Feature Scope
//!
//! Provides aggregated statistics, operational health, and module metrics
//! for the Master Data Management platform.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::*;
pub use errors::*;
pub use models::*;
pub use queries::*;
