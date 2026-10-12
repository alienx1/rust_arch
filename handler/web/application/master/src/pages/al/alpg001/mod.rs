//! ALPG001 Page Module (Process Monitor: background processes started through Redpanda, their
//! outcome and a "reprocess" action; Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::Alpg001Screen as Alpg001Page;
