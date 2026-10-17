//! Contacts action: the contacts of a record

use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::contact_executor::ContactCommandExecutor;
use crate::features::db::contacts::errors::ContactError;
use crate::features::db::contacts::models::{ContactDto, ContactOwner};
use domain::entity::{db_branch, db_company, db_contact, db_employee, db_master_lookup, db_organization};

impl ContactCommandExecutor {
    /// The contact group of the record: `NotFound` when the record is missing, `None` when it has
    /// no group yet
    pub(super) async fn group_of(&self, owner: ContactOwner, id: Uuid) -> Result<Option<Uuid>, ContactError> {
        let missing = || ContactError::NotFound(format!("{id} not found"));
        Ok(match owner {
            ContactOwner::Organization => {
                db_organization::Entity::find_by_id(id).one(&self.db).await?.ok_or_else(missing)?.contact_group_id
            }
            ContactOwner::Company => {
                db_company::Entity::find_by_id(id).one(&self.db).await?.ok_or_else(missing)?.contact_group_id
            }
            ContactOwner::Branch => {
                Some(db_branch::Entity::find_by_id(id).one(&self.db).await?.ok_or_else(missing)?.contact_group_id)
            }
            ContactOwner::Employee => {
                db_employee::Entity::find_by_id(id).one(&self.db).await?.ok_or_else(missing)?.contact_group_id
            }
        })
    }

    pub(super) async fn list_contacts(&self, owner: ContactOwner, id: Uuid) -> Result<Vec<ContactDto>, ContactError> {
        let Some(group) = self.group_of(owner, id).await? else {
            return Ok(Vec::new());
        };
        let rows = db_contact::Entity::find()
            .filter(db_contact::Column::ContactGroupId.eq(group))
            .order_by_asc(db_contact::Column::Id)
            .all(&self.db)
            .await?;
        let codes: HashMap<Uuid, String> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("contact_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| (m.id, m.code))
            .collect();
        let mut contacts: Vec<ContactDto> = rows
            .into_iter()
            .filter_map(|r| {
                codes.get(&r.master_lookup_id).map(|code| ContactDto {
                    type_code: code.clone(),
                    value: r.value,
                    default_contact: r.default_contact,
                })
            })
            .collect();
        // Stable order: by kind, the default of each kind first
        contacts.sort_by(|a, b| (&a.type_code, !a.default_contact).cmp(&(&b.type_code, !b.default_contact)));
        Ok(contacts)
    }
}
