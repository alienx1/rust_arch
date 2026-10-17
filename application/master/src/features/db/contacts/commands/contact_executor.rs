//! Contact executor: routes each `ContactCommands` method to its action file

use async_trait::async_trait;
use std::sync::Arc;

use application_core::traits::IdentityProvisioningPort;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::ContactCommands;
use crate::features::db::contacts::errors::ContactError;
use crate::features::db::contacts::models::{ContactDto, ContactInput, ContactOwner};

pub struct ContactCommandExecutor {
    pub(super) db: DatabaseConnection,
    /// Where the sign-in e-mail of an employee's account is kept in step with their e-mail contact
    pub(super) identity: Option<Arc<dyn IdentityProvisioningPort>>,
}

impl ContactCommandExecutor {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, identity: None }
    }

    /// Keeps the account's sign-in e-mail (Kratos `traits.email`) equal to the employee's default
    /// e-mail contact whenever the contacts are replaced
    pub fn with_identity(mut self, identity: Arc<dyn IdentityProvisioningPort>) -> Self {
        self.identity = Some(identity);
        self
    }
}

#[async_trait]
impl ContactCommands for ContactCommandExecutor {
    async fn list_contacts(&self, owner: ContactOwner, id: Uuid) -> Result<Vec<ContactDto>, ContactError> {
        self.list_contacts(owner, id).await
    }

    async fn replace_contacts(
        &self,
        owner: ContactOwner,
        id: Uuid,
        contacts: Vec<ContactInput>,
        actor_id: Option<Uuid>,
    ) -> Result<Vec<ContactDto>, ContactError> {
        self.replace_contacts(owner, id, contacts, actor_id).await
    }
}
