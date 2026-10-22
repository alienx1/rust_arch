//! DBPG005 (geography) commands

use application_master::features::db::dbpg005::{DbDbpg005Queries, Dbpg005CommandExecutor, Dbpg005Commands, Dbpg005Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg005Queries> = Arc::new(DbDbpg005Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg005Commands> =
        Arc::new(Dbpg005CommandExecutor::new(queries, cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(commands);
}
