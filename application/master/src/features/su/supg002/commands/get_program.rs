//! SUPG002 get one program (delegates to `Supg002Queries`)

use uuid::Uuid;

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::ProgramDto;

impl Supg002CommandExecutor {
    pub(super) async fn get_program(&self, id: Uuid) -> Result<Option<ProgramDto>, Supg002Error> {
        self.queries.get_program(id).await
    }
}
