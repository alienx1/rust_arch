//! SUPG006 — Security & Mail Settings (ตั้งค่าความปลอดภัยและอีเมล): the sign-in brute-force lock
//! (`security.*`, the password policy) and the password e-mail templates (`mail.template.*`) kept in `su_configuration`.
//!
//! Program code `SUPG006`, API `/api/v1/su/supg006/config`. Only the keys listed in
//! [`models::CONFIG_SETTINGS`] can be read or written here; the SMTP settings (`mail.smtp_url`,
//! `mail.from`) hold a password and are deliberately not exposed. Rows are saved with optimistic
//! locking (`row_version`); the account security feature reads them through a one-minute cache that
//! a write to this program clears.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{Supg006CommandExecutor, Supg006Commands, validate_change};
pub use errors::Supg006Error;
pub use models::*;
pub use queries::{DbSupg006Queries, Supg006Queries};
