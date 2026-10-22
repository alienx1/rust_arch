//! ALPG000 (audit log) commands: search over the audit store, client event intake, and CSV
//! export requests that go to the report worker through Redpanda

use application_master::features::al::alpg000::{Alpg000CommandExecutor, Alpg000Commands, DbAlpg000Queries};
use std::sync::Arc;
use tracing::warn;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let mut executor = Alpg000CommandExecutor::new(cx.audit_log.clone())
        .with_cache(cx.caches.clone())
        .with_programs(Arc::new(DbAlpg000Queries::new(cx.db.clone())));
    // Creating the producer does not connect: a broker that is down only fails the request
    match &cx.broker {
        Some(events) => executor = executor.with_events(events.clone()),
        None => warn!("Report queue (Redpanda) not available"),
    }
    let commands: Arc<dyn Alpg000Commands> = Arc::new(executor);
    services.insert(commands);
}
