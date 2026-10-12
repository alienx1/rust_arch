//! List postal codes (delegates to `Dbpg005Queries`)

use super::dbpg005_executor::Dbpg005CommandExecutor;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{PostalFilterQuery, PostalPageDto};

impl Dbpg005CommandExecutor {
    pub(super) async fn list_postal_codes(
        &self,
        filter: PostalFilterQuery,
    ) -> Result<PostalPageDto, Dbpg005Error> {
        self.queries.list_postal_codes(filter).await
    }
}
