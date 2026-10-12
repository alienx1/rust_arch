//! DBPG004 Page Module (Reference Data: paged list + create/edit form, Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::{Dbpg004LookupScreen as Dbpg004LookupPage, Dbpg004Screen as Dbpg004Page};
