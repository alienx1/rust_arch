//! DBPG002 write: hard-delete a branch with its owned labels, address group and contact group

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use crate::features::db::dbpg002::commands::dbpg002_executor::Dbpg002CommandExecutor;
use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::db::dbpg002::models::{
    db_address_group, db_branch, db_branch_label, db_contact_group,
};
use crate::features::referential_integrity::is_foreign_key_violation;
use domain::entity::{db_address, db_contact};

impl Dbpg002CommandExecutor {
    pub(super) async fn delete_branch(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        let existing = db_branch::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;
        if self.head_type_ids().await?.contains(&existing.master_lookup_id) {
            return Err(Dbpg002Error::Validation(
                "The headquarter branch cannot be deleted".to_string(),
            ));
        }

        let in_use = || {
            Dbpg002Error::InUse(
                "This branch is still referenced by other records; switch it off instead"
                    .to_string(),
            )
        };

        // Owned data first; the branch row goes before its groups (it holds the group FKs)
        let txn = self.db.begin().await?;
        db_branch_label::Entity::delete_many()
            .filter(db_branch_label::Column::BranchId.eq(id))
            .exec(&txn)
            .await?;
        if let Err(err) = db_branch::Entity::delete_by_id(id).exec(&txn).await {
            return Err(if is_foreign_key_violation(&err) { in_use() } else { err.into() });
        }
        db_address::Entity::delete_many()
            .filter(db_address::Column::AddressGroupId.eq(existing.address_group_id))
            .exec(&txn)
            .await?;
        db_contact::Entity::delete_many()
            .filter(db_contact::Column::ContactGroupId.eq(existing.contact_group_id))
            .exec(&txn)
            .await?;
        db_address_group::Entity::delete_by_id(existing.address_group_id)
            .exec(&txn)
            .await?;
        db_contact_group::Entity::delete_by_id(existing.contact_group_id)
            .exec(&txn)
            .await?;
        txn.commit().await?;
        Ok(())
    }
}
