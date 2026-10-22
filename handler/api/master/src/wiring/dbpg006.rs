//! DBPG006 (i18n labels) commands

use application_master::features::db::dbpg006::{DbDbpg006Queries, Dbpg006CommandExecutor, Dbpg006Commands, Dbpg006Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg006Queries> = Arc::new(DbDbpg006Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg006Commands> =
        Arc::new(Dbpg006CommandExecutor::new(queries, cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(commands);
}
