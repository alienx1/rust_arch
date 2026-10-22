//! DBPG003 (employees) commands: switching an employee off suspends their accounts (SUPG005),
//! photos live in S3

use application_master::features::db::dbpg003::{DbDbpg003Queries, Dbpg003CommandExecutor, Dbpg003Commands, Dbpg003Queries};
use application_master::features::su::supg005::Supg005Commands;
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Dbpg003Queries> = Arc::new(DbDbpg003Queries::new(cx.db.clone()));
    let commands: Arc<dyn Dbpg003Commands> = Arc::new(
        Dbpg003CommandExecutor::new(queries, cx.db.clone())
            .with_cache(cx.caches.clone())
            .with_account_suspension(services.require::<dyn Supg005Commands>())
            .with_images(cx.image_content.clone(), cx.image_bucket.clone()),
    );
    services.insert(commands);
}
