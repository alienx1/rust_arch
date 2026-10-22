//! Wiring of api-master: how each feature's command interface is built and registered.
//!
//! One file per feature (`wiring/<feature>.rs`, a `register(cx, services)` function that builds the
//! executor from the shared [`WiringContext`] and registers it by type), and one line per feature in
//! [`register_all`]. A new program adds its file and its line here; `connection.rs` and the routers'
//! `State<Arc<dyn ...>>` extraction need no change (see `services.rs`).
//!
//! Order matters only where a feature builds on another: `Supg005` before `Dbpg003` and `Supg004`,
//! `AccountSecurity` before `Supg004` (`ServicesBuilder::require` panics at start-up otherwise).

pub mod access_control;
pub mod account_security;
pub mod alpg000;
pub mod alpg001;
pub mod contacts;
pub mod dashboard;
pub mod dbpg000;
pub mod dbpg001;
pub mod dbpg002;
pub mod dbpg003;
pub mod dbpg004;
pub mod dbpg005;
pub mod dbpg006;
pub mod localization;
pub mod supg001;
pub mod supg002;
pub mod supg003;
pub mod supg004;
pub mod supg005;
pub mod supg006;

use application_core::traits::{AuditLogPort, ContentService, ReadCaches};
use application_master::features::localization::LocalizationQueries;
use infrastructure::middleware::KratosConfig;
use infrastructure::services::{RedpandaEventService, ValkeyCacheService};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::services::{Services, ServicesBuilder};

/// What the features are built from (the shared infrastructure of the process)
pub struct WiringContext {
    pub db: DatabaseConnection,
    /// Valkey connection, `None` when it is unreachable
    pub cache: Option<ValkeyCacheService>,
    /// Read-through caches (disabled without Valkey)
    pub caches: ReadCaches,
    /// Ory Kratos (identity provider)
    pub kratos: Option<KratosConfig>,
    pub audit_log: Arc<dyn AuditLogPort>,
    /// Redpanda client (report requests, mail, notices); `None` when it cannot be created
    pub broker: Option<Arc<RedpandaEventService>>,
    /// Garage / S3 for logos and photos
    pub image_content: Arc<dyn ContentService>,
    pub image_bucket: String,
    /// Dictionary reads (cached), shared by the localization feature and the error templates
    pub loc_queries: Arc<dyn LocalizationQueries>,
}

/// Registers every feature, in dependency order
pub fn register_all(cx: &WiringContext) -> Services {
    let mut s = ServicesBuilder::default();
    localization::register(cx, &mut s);
    dashboard::register(cx, &mut s);
    access_control::register(cx, &mut s);
    supg005::register(cx, &mut s);
    dbpg000::register(cx, &mut s);
    dbpg001::register(cx, &mut s);
    contacts::register(cx, &mut s);
    dbpg002::register(cx, &mut s);
    dbpg003::register(cx, &mut s);
    dbpg004::register(cx, &mut s);
    dbpg005::register(cx, &mut s);
    dbpg006::register(cx, &mut s);
    supg001::register(cx, &mut s);
    supg002::register(cx, &mut s);
    supg003::register(cx, &mut s);
    supg006::register(cx, &mut s);
    account_security::register(cx, &mut s);
    supg004::register(cx, &mut s);
    alpg000::register(cx, &mut s);
    alpg001::register(cx, &mut s);
    s.build()
}
