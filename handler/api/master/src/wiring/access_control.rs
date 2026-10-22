//! Permission checks of the program APIs

use application_master::features::su::access_control::{AccessControl, DbAccessControl};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let access: Arc<dyn AccessControl> = Arc::new(DbAccessControl::new(cx.db.clone()).with_cache(cx.caches.clone()));
    services.insert(access);
}
