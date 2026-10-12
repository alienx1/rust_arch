//! SUPG003 — Permission Matrix (แฟ้มสิทธิ์การเข้าใช้งาน): permission groups and the
//! program × action matrix (`su_permission_group`, `su_permission`).
//!
//! Program code `SUPG003`, API `/api/v1/su/supg003`. A group's matrix decides which programs its
//! users see in the menu (READ) and may change (WRITE / APPROVE).

pub mod commands;
pub mod errors;
pub mod models;
pub mod queries;

pub use commands::{
    ChangePlan, ExistingGrant, Supg003CommandExecutor, Supg003Commands, keeps_own_access,
    normalize_group_code, plan_changes,
};
pub use errors::Supg003Error;
pub use models::*;
pub use queries::{DbSupg003Queries, Supg003Queries, order_programs};
