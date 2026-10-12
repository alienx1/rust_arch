//! Get one postal code (delegates to `Dbpg005Queries`)

use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::PostalCodeDto;

impl Dbpg005CommandExecutor {
    pub(super) async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error> {
        self.queries.get_postal_code(id).await
    }
}
