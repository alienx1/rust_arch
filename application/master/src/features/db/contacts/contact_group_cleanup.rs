//! Removing a contact group together with its contacts, for owners that are being deleted

use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use domain::entity::{db_contact, db_contact_group};

/// Deletes the contacts of `group_id` and then the group. Call it after the owner row is gone: the
/// owner points at the group.
pub async fn delete_contact_group<C: ConnectionTrait>(db: &C, group_id: Uuid) -> Result<(), DbErr> {
    db_contact::Entity::delete_many().filter(db_contact::Column::ContactGroupId.eq(group_id)).exec(db).await?;
    db_contact_group::Entity::delete_by_id(group_id).exec(db).await?;
    Ok(())
}
