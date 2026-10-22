//! DBPG001 (organization & company create / update, logos in S3) commands

use application_master::features::db::dbpg000::{DbDbpg000Queries, Dbpg000Queries};
use application_master::features::db::dbpg001::{DbDbpg001Queries, Dbpg001CommandExecutor, Dbpg001Commands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    // Shares DBPG000's read runner (a thin wrapper over the same connection)
    let read_runner: Arc<dyn Dbpg000Queries> = Arc::new(DbDbpg000Queries::new(cx.db.clone()));
    let queries = Arc::new(DbDbpg001Queries::new(read_runner));
    let commands: Arc<dyn Dbpg001Commands> = Arc::new(
        Dbpg001CommandExecutor::new(queries, cx.db.clone())
            .with_cache(cx.caches.clone())
            .with_images(cx.image_content.clone(), cx.image_bucket.clone()),
    );
    services.insert(commands);
}
