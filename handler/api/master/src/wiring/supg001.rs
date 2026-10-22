//! SUPG001 (systems & modules) commands

use application_master::features::su::supg001::{DbSupg001Queries, Supg001CommandExecutor, Supg001Commands, Supg001Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Supg001Queries> = Arc::new(DbSupg001Queries::new(cx.db.clone()));
    let commands: Arc<dyn Supg001Commands> = Arc::new(Supg001CommandExecutor::new(queries, cx.db.clone()));
    services.insert(commands);
}
