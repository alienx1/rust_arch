//! Demo UI Showcase Module
//!
//! Organized into Clean UI architecture:
//! - `models`: Demo DTOs and state types
//! - `services`: Static mock data and environment detection
//! - `providers`: Interactive demo state context
//! - `components`: Modular showcase presentations
//! - `screens`: Full demo screen layout

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::DemoScreen as DemoPage;
pub use services::is_dev_env;
