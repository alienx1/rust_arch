//! SUPG002 (menu tree / programs) commands

use application_master::features::su::supg002::{DbSupg002Queries, Supg002CommandExecutor, Supg002Commands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries = Arc::new(DbSupg002Queries::new(cx.db.clone()));
    let commands: Arc<dyn Supg002Commands> =
        Arc::new(Supg002CommandExecutor::new(queries, cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(commands);
}
