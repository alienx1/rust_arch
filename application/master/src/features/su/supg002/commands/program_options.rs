//! SUPG002 form selector options (delegates to `Supg002Queries`)

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::ProgramOptionsDto;

impl Supg002CommandExecutor {
    pub(super) async fn program_options(&self) -> Result<ProgramOptionsDto, Supg002Error> {
        self.queries.program_options().await
    }
}
