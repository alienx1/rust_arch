//! Contacts action: replace all contacts of a record

use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

use super::contact_executor::ContactCommandExecutor;
use crate::features::db::contacts::errors::ContactError;
use crate::features::db::contacts::models::{ContactDto, ContactInput, ContactOwner, normalize_contacts};
use application_core::traits::IdentityError;
use domain::entity::su_user;
use domain::entity::{db_company, db_contact, db_contact_group, db_master_lookup, db_organization};

impl ContactCommandExecutor {
    pub(super) async fn replace_contacts(
        &self,
        owner: ContactOwner,
        id: Uuid,
        contacts: Vec<ContactInput>,
        actor_id: Option<Uuid>,
    ) -> Result<Vec<ContactDto>, ContactError> {
        let contacts = normalize_contacts(owner, contacts)?;
        let group = self.group_of(owner, id).await?;
        if group.is_none() && contacts.is_empty() {
            return Ok(Vec::new());
        }

        let lookups: HashMap<String, Uuid> = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq("contact_type"))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| (m.code, m.id))
            .collect();

        let txn = self.db.begin().await?;
        let group_id = match group {
            Some(group) => group,
            None => {
                let group_id = Uuid::now_v7();
                db_contact_group::ActiveModel {
                    id: Set(group_id),
                    active: Set(Some(true)),
                    change_by: Set(actor_id),
                    row_version: Set(1),
                }
                .insert(&txn)
                .await?;
                // Branches and employees get their group elsewhere; organizations and companies
                // start without one and receive it here (the row version stays: see the trait)
                match owner {
                    ContactOwner::Organization => {
                        let mut row = db_organization::ActiveModel { id: Set(id), ..Default::default() };
                        row.contact_group_id = Set(Some(group_id));
                        row.update(&txn).await?;
                    }
                    ContactOwner::Company => {
                        let mut row = db_company::ActiveModel { id: Set(id), ..Default::default() };
                        row.contact_group_id = Set(Some(group_id));
                        row.update(&txn).await?;
                    }
                    ContactOwner::Employee => {
                        let mut row = domain::entity::db_employee::ActiveModel { id: Set(id), ..Default::default() };
                        row.contact_group_id = Set(Some(group_id));
                        row.update(&txn).await?;
                    }
                    ContactOwner::Branch => unreachable!("a branch always has its contact group"),
                }
                group_id
            }
        };

        db_contact::Entity::delete_many()
            .filter(db_contact::Column::ContactGroupId.eq(group_id))
            .exec(&txn)
            .await?;
        for c in &contacts {
            let lookup = lookups
                .get(&c.type_code)
                .copied()
                .ok_or_else(|| ContactError::Validation(format!("Contact type {} is not set up in the database", c.type_code)))?;
            db_contact::ActiveModel {
                id: Set(Uuid::now_v7()),
                contact_group_id: Set(group_id),
                master_lookup_id: Set(lookup),
                value: Set(c.value.clone()),
                default_contact: Set(c.default_contact),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(&txn)
            .await?;
        }

        // The employee's account signs in with their default e-mail: change it in Kratos before the
        // contacts are committed, so a refused address (already used) leaves everything as it was
        let undo = self.sync_sign_in_email(owner, id, &contacts).await?;
        if let Err(e) = txn.commit().await {
            if let Some((login, old)) = undo
                && let Some(identity) = &self.identity
                && let Err(undo_err) = identity.update_email(&login, &old).await
            {
                tracing::error!(account = %login, "sign-in e-mail not restored after a failed save: {undo_err}");
            }
            return Err(e.into());
        }
        self.list_contacts(owner, id).await
    }

    /// Sets the sign-in e-mail of the employee's account to their default e-mail contact. Returns
    /// `(user name, previous e-mail)` when it was changed, so the caller can undo it.
    async fn sync_sign_in_email(
        &self,
        owner: ContactOwner,
        id: Uuid,
        contacts: &[ContactDto],
    ) -> Result<Option<(String, String)>, ContactError> {
        let Some(identity) = (owner == ContactOwner::Employee).then_some(self.identity.as_ref()).flatten() else {
            return Ok(None);
        };
        let Some(email) = contacts
            .iter()
            .filter(|c| c.type_code == "EMAIL")
            .max_by_key(|c| c.default_contact)
            .map(|c| c.value.trim().to_string())
        else {
            return Ok(None);
        };
        let Some(account) = su_user::Entity::find().filter(su_user::Column::EmployeeId.eq(id)).one(&self.db).await? else {
            return Ok(None);
        };
        let previous = match identity.find_email(&account.username).await {
            Ok(Some(old)) if old.eq_ignore_ascii_case(&email) => return Ok(None),
            Ok(Some(old)) => old,
            Ok(None) => return Ok(None), // the account has no identity to keep in step
            Err(e) => return Err(ContactError::Identity(e.to_string())),
        };
        match identity.update_email(&account.username, &email).await {
            Ok(()) => {
                tracing::info!(account = %account.username, "sign-in e-mail changed with the employee's contacts");
                Ok(Some((account.username, previous)))
            }
            Err(IdentityError::Conflict(_)) => Err(ContactError::Conflict(format!("The e-mail {email} is already used by another account"))),
            Err(IdentityError::NotFound(_)) => Ok(None),
            Err(e) => Err(ContactError::Identity(e.to_string())),
        }
    }
}
