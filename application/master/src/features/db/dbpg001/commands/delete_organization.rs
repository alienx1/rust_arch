//! DBPG001 command: delete an organization that nothing references

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::features::db::dbpg001::errors::Dbpg001Error;
use crate::features::db::dbpg001::models::{db_organization, db_organization_label};
use crate::features::referential_integrity::is_foreign_key_violation;

use super::dbpg001_executor::Dbpg001CommandExecutor;

impl Dbpg001CommandExecutor {
    /// Deletes the organization's labels and then the row in one transaction; a foreign-key violation
    /// (still referenced) rolls everything back and reports `InUse`.
    pub(super) async fn delete_organization(&self, id: Uuid) -> Result<(), Dbpg001Error> {
        let in_use = |err: sea_orm::DbErr| {
            if is_foreign_key_violation(&err) {
                Dbpg001Error::InUse(
                    "This organization is still referenced by other records and cannot be deleted; switch it off instead"
                        .to_string(),
                )
            } else {
                Dbpg001Error::from(err)
            }
        };

        let txn = self.db.begin().await?;
        let existing = db_organization::Entity::find_by_id(id).one(&txn).await?;
        if existing.is_none() {
            return Err(Dbpg001Error::NotFound(id.to_string()));
        }

        db_organization_label::Entity::delete_many()
            .filter(db_organization_label::Column::OrganizationId.eq(id))
            .exec(&txn)
            .await
            .map_err(in_use)?;
        db_organization::Entity::delete_by_id(id)
            .exec(&txn)
            .await
            .map_err(in_use)?;
        txn.commit().await.map_err(in_use)?;
        Ok(())
    }
}
