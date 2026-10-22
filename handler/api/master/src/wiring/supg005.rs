//! SUPG005 (IAM & sessions) commands. Other features suspend / reactivate accounts through it.

use application_master::features::su::supg005::{DbSupg005Queries, Supg005CommandExecutor, Supg005Commands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries = Arc::new(DbSupg005Queries::new(cx.db.clone()));
    let mut executor = Supg005CommandExecutor::new(queries);
    if let Some(k) = &cx.kratos {
        executor = executor.with_revocation(Some(Arc::new(k.clone())));
    }
    let commands: Arc<dyn Supg005Commands> = Arc::new(executor);
    services.insert(commands);
}
