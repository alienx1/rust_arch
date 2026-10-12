//! DBPG004 command: retrieve a single lookup by ID

use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::LookupDto;

use super::dbpg004_executor::Dbpg004CommandExecutor;

impl Dbpg004CommandExecutor {
    pub(super) async fn get_lookup_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<LookupDto>, Dbpg004Error> {
        self.queries.get_lookup_by_id(id).await
    }
}
