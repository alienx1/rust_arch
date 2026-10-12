//! IAM Session Feature Module
//!
//! Organized into Clean UI architecture:
//! - `models`: DTOs and view models
//! - `services`: Kratos and IAM API service integration
//! - `providers`: Reactive context and state management
//! - `components`: Modular, reusable presentation components
//! - `screens`: Page-level screen orchestrator

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::Supg005Screen as Supg005Page;
