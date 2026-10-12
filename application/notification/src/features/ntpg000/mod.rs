//! NTPG000 — Notifications (การแจ้งเตือน): what the system tells each user.
//!
//! Any service publishes a [`NotificationMessage`](application_core::traits::NotificationMessage)
//! to Redpanda. [`Ntpg000Commands::ingest`] stores it and pushes it to the recipient's open
//! sessions; the same feature serves the user's list, the unread count and "mark as read".

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{NotificationMessageHandler, Ntpg000CommandExecutor, Ntpg000Commands};
pub use errors::Ntpg000Error;
pub use models::*;
pub use queries::{DbNtpg000Queries, Ntpg000Queries};
