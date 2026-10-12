//! SUPG001 get one system (delegates to `Supg001Queries`)

use uuid::Uuid;

use super::supg001_executor::Supg001CommandExecutor;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::SystemDto;

impl Supg001CommandExecutor {
    pub(super) async fn get_system(&self, id: Uuid) -> Result<Option<SystemDto>, Supg001Error> {
        self.queries.get_system(id).await
    }
}
