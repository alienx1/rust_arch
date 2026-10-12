//! SUPG001 get one module (delegates to `Supg001Queries`)

use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::ModuleDto;

impl Supg001CommandExecutor {
    pub(super) async fn get_module(&self, id: Uuid) -> Result<Option<ModuleDto>, Supg001Error> {
        self.queries.get_module(id).await
    }
}
