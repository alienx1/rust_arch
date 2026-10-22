//! DBPG004 (reference data) commands

use application_master::features::db::dbpg004::{DbDbpg004Queries, Dbpg004CommandExecutor, Dbpg004Commands, Dbpg004Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg004Queries> = Arc::new(DbDbpg004Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg004Commands> =
        Arc::new(Dbpg004CommandExecutor::new(queries, cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(commands);
}
