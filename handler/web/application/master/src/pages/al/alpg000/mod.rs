//! ALPG000 Page Module (Audit Log: user access + API system logs from Quickwit, Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::{Alpg000LogScreen as Alpg000LogPage, Alpg000Screen as Alpg000Page};
