//! SUPG002 list programs (delegates to `Supg002Queries`)

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::{ProgramFilterQuery, ProgramPageDto};

impl Supg002CommandExecutor {
    pub(super) async fn list_programs(&self, filter: ProgramFilterQuery) -> Result<ProgramPageDto, Supg002Error> {
        self.queries.list_programs(filter).await
    }
}
