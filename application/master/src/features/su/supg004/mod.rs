//! SUPG004 — User Accounts (กำหนดผู้ใช้งาน): `su_user` plus its Ory Kratos identity.
//!
//! Program code `SUPG004`, API `/api/v1/su/supg004`. Creating an account creates the Kratos
//! identity (no password: the holder sets one through a one-time recovery link), then the local
//! `su_user` row; switching an account off also blocks login and revokes its sessions at Kratos.

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{
    Supg004CommandExecutor, Supg004Commands, is_valid_email, normalize_assignments, normalize_username,
};
pub use errors::Supg004Error;
pub use models::*;
pub use queries::{DbSupg004Queries, Supg004Queries};
