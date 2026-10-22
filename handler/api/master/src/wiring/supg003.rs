//! SUPG003 (permission matrix) commands

use application_master::features::su::supg003::{DbSupg003Queries, Supg003CommandExecutor, Supg003Commands, Supg003Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Supg003Queries> = Arc::new(DbSupg003Queries::new(cx.db.clone()));
    let commands: Arc<dyn Supg003Commands> = Arc::new(Supg003CommandExecutor::new(queries, cx.db.clone()));
    services.insert(commands);
}
