//! SUPG004 permission-group lookup and identity access shared by the account actions.

use application_core::traits::IdentityProvisioningPort;
use sea_orm::EntityTrait;
use std::sync::Arc;
use uuid::Uuid;

use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::supg004::errors::Supg004Error;
use domain::entity::su_permission_group;

impl Supg004CommandExecutor {
    pub(super) fn identity(&self) -> Result<&Arc<dyn IdentityProvisioningPort>, Supg004Error> {
        self.identity
            .as_ref()
            .ok_or_else(|| Supg004Error::IdentityProvider("The identity provider is not configured".to_string()))
    }

    pub(super) async fn group(&self, id: Uuid) -> Result<su_permission_group::Model, Supg004Error> {
        su_permission_group::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg004Error::NotFound(format!("Permission group '{id}' does not exist")))
    }
}
