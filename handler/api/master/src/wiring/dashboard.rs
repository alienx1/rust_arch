//! Dashboard commands

use application_master::features::dashboard::{DashboardCommandExecutor, DashboardCommands, DbDashboardQueries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries = Arc::new(DbDashboardQueries::new(cx.db.clone()));
    let commands: Arc<dyn DashboardCommands> = Arc::new(DashboardCommandExecutor::new(queries).with_cache(cx.caches.clone()));
    services.insert(commands);
}
