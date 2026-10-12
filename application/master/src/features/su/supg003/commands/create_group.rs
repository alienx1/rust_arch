//! SUPG003 create-group action: validates the code (unique) and inserts the group.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use super::supg003_executor::Supg003CommandExecutor;
use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    CreatePermissionGroupCommand, PermissionGroupDto, su_permission_group,
};

/// Group codes are 2–50 upper-case letters, digits or underscores (`ADMIN`, `HR_MANAGER`).
pub fn normalize_group_code(raw: &str) -> Result<String, Supg003Error> {
    let c = raw.trim().to_uppercase();
    let valid = (2..=50).contains(&c.len())
        && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    if !valid {
        return Err(Supg003Error::Validation(
            "Code must be 2-50 letters, digits or underscores (e.g. HR_MANAGER)".to_string(),
        ));
    }
    Ok(c)
}

impl Supg003CommandExecutor {
    pub(super) async fn create_group(
        &self,
        cmd: CreatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error> {
        let code = normalize_group_code(&cmd.code)?;
        let taken = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Code.eq(code.clone()))
            .count(&self.db)
            .await?;
        if taken > 0 {
            return Err(Supg003Error::DuplicateCode(format!("Group code '{code}' already exists")));
        }

        let id = Uuid::now_v7();
        info!("Creating new permission group: id={id}, code={code}");
        su_permission_group::ActiveModel {
            id: Set(id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&self.db)
        .await?;

        self.queries
            .get_group(id)
            .await?
            .ok_or_else(|| Supg003Error::Internal("Failed to retrieve created group".to_string()))
    }
}
