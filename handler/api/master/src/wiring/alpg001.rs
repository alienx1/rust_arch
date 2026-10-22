//! ALPG001 (process monitor): reads the report job rows and re-publishes their requests

use application_master::features::al::alpg001::{Alpg001CommandExecutor, Alpg001Commands, DbAlpg001Queries};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let mut executor = Alpg001CommandExecutor::new(Arc::new(DbAlpg001Queries::new(cx.db.clone())));
    if let Some(events) = &cx.broker {
        executor = executor.with_events(events.clone());
    }
    let commands: Arc<dyn Alpg001Commands> = Arc::new(executor);
    services.insert(commands);
}
