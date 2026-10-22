//! SUPG006 (security & mail settings) commands

use application_master::features::su::supg006::{DbSupg006Queries, Supg006CommandExecutor, Supg006Commands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let commands: Arc<dyn Supg006Commands> =
        Arc::new(Supg006CommandExecutor::new(Arc::new(DbSupg006Queries::new(cx.db.clone())), cx.db.clone()));
    services.insert(commands);
}
