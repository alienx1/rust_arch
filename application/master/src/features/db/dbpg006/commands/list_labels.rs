//! DBPG006 command: list one page of label entries matching the filter

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{LabelFilterQuery, LabelPageDto};

use super::dbpg006_executor::Dbpg006CommandExecutor;

impl Dbpg006CommandExecutor {
    pub(super) async fn list_labels(
        &self,
        filter: LabelFilterQuery,
    ) -> Result<LabelPageDto, Dbpg006Error> {
        self.queries.list_labels(filter).await
    }
}
