//! SUPG001 list modules (delegates to `Supg001Queries`)

use super::supg001_executor::Supg001CommandExecutor;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{ModuleDto, ModuleFilterQuery};

impl Supg001CommandExecutor {
    pub(super) async fn list_modules(&self, filter: ModuleFilterQuery) -> Result<Vec<ModuleDto>, Supg001Error> {
        self.queries.list_modules(filter).await
    }
}
