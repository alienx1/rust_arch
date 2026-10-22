//! DBPG002 (branches & departments) commands

use application_master::features::db::dbpg002::{DbDbpg002Queries, Dbpg002CommandExecutor, Dbpg002Commands, Dbpg002Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg002Queries> = Arc::new(DbDbpg002Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg002Commands> =
        Arc::new(Dbpg002CommandExecutor::new(queries, cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(commands);
}
