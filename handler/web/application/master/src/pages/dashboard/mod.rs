//! Dashboard Feature Module for Master Web Application
//!
//! Organized into CQRS & Clean UI architecture:
//! - `models`: DTOs and data representations
//! - `services`: HTTP API client integration
//! - `providers`: Reactive state and context management
//! - `components`: Modular, reusable UI presentation components
//! - `screens`: Page-level composition screens

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::DashboardScreen as DashboardPage;
