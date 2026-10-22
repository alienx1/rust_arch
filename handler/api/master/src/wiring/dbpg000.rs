//! DBPG000 (organizations & companies list) commands

use application_master::features::db::dbpg000::{DbDbpg000Queries, Dbpg000CommandExecutor, Dbpg000Commands, Dbpg000Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg000Queries> = Arc::new(DbDbpg000Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg000Commands> = Arc::new(Dbpg000CommandExecutor::new(queries).with_cache(cx.caches.clone()));
    services.insert(commands);
}
