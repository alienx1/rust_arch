//! SUPG001 list systems (delegates to `Supg001Queries`)

use super::supg001_executor::Supg001CommandExecutor;
use crate::features::su::supg001::errors::Supg001Error;
use crate::features::su::supg001::models::{SystemDto, SystemFilterQuery};

impl Supg001CommandExecutor {
    pub(super) async fn list_systems(&self, filter: SystemFilterQuery) -> Result<Vec<SystemDto>, Supg001Error> {
        self.queries.list_systems(filter).await
    }
}
