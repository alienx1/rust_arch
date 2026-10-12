//! DBPG002 branch type rules: valid type id and one headquarter per company

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{BRANCH_TYPE_HEAD, db_branch};
use domain::entity::db_master_lookup;

impl Dbpg002CommandExecutor {
    /// Ids of every `branch_type` lookup row whose code is HEAD (the lookup table may repeat rows).
    pub(super) async fn head_type_ids(&self) -> Result<Vec<Uuid>, Dbpg002Error> {
        Ok(db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("branch_type"))
            .filter(db_master_lookup::Column::Code.eq(BRANCH_TYPE_HEAD))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|l| l.id)
            .collect())
    }

    /// Validates the branch type id and rejects a second headquarter for the company.
    pub(super) async fn check_branch_type(
        &self,
        company_id: Uuid,
        master_lookup_id: Uuid,
        exclude_branch: Option<Uuid>,
    ) -> Result<(), Dbpg002Error> {
        let lookup = db_master_lookup::Entity::find_by_id(master_lookup_id)
            .filter(db_master_lookup::Column::Category.eq("branch_type"))
            .one(&self.db)
            .await?;
        if lookup.is_none() {
            return Err(Dbpg002Error::Validation(format!(
                "Branch type '{master_lookup_id}' does not exist"
            )));
        }
        let head_ids = self.head_type_ids().await?;
        if !head_ids.contains(&master_lookup_id) {
            return Ok(());
        }
        let mut select = db_branch::Entity::find()
            .filter(db_branch::Column::CompanyId.eq(company_id))
            .filter(db_branch::Column::MasterLookupId.is_in(head_ids));
        if let Some(id) = exclude_branch {
            select = select.filter(db_branch::Column::Id.ne(id));
        }
        if select.one(&self.db).await?.is_some() {
            return Err(Dbpg002Error::Validation(
                "This company already has a headquarter branch".to_string(),
            ));
        }
        Ok(())
    }
}
