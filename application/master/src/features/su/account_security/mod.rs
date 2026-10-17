//! Account security: passwords by e-mail, forced password change and the sign-in lock.
//!
//! - A new account, a reset and a forced change all end the same way: a one-time link is e-mailed
//!   to the account holder, who sets the password at the identity provider. The application never
//!   handles a password.
//! - `su_user.must_change_password` keeps an account out until that has happened: the identity
//!   provider's login hook refuses it (and e-mails the link again, at most every few minutes) and
//!   its settings hook clears the flag once a new password is set.
//! - Too many failed sign-ins within a window lock the account for a while. The numbers live in
//!   `su_configuration` (category `security`), the counters in Valkey.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{AccountSecurityCommands, AccountSecurityExecutor};
pub use errors::AccountSecurityError;
pub use models::*;
pub use queries::{AccountSecurityQueries, DbAccountSecurityQueries};
