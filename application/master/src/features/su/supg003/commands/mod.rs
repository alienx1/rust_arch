//! SUPG003 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::su::supg003::errors::Supg003Error;
use crate::features::su::supg003::models::{
    CreatePermissionGroupCommand, GroupFilterQuery, PermissionGroupDto, PermissionMatrixDto,
    SaveMatrixCommand, UpdatePermissionGroupCommand,
};

mod create_group;
mod delete_group;
mod save_matrix;
pub mod supg003_executor;
mod update_group;

pub use create_group::normalize_group_code;
pub use save_matrix::{ChangePlan, ExistingGrant, keeps_own_access, plan_changes};
pub use supg003_executor::Supg003CommandExecutor;

#[async_trait]
pub trait Supg003Commands: Send + Sync {
    async fn list_groups(&self, filter: GroupFilterQuery) -> Result<Vec<PermissionGroupDto>, Supg003Error>;
    async fn get_group(&self, id: Uuid) -> Result<Option<PermissionGroupDto>, Supg003Error>;
    async fn group_matrix(&self, id: Uuid) -> Result<Option<PermissionMatrixDto>, Supg003Error>;

    /// Create a group (code + active). A group has no scope: scopes belong to a user's assignments
    async fn create_group(
        &self,
        cmd: CreatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error>;

    /// Switch a group on / off (optimistic lock). `ADMIN` cannot be switched off and a group that
    /// still has users cannot be switched off.
    async fn update_group(
        &self,
        id: Uuid,
        cmd: UpdatePermissionGroupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionGroupDto, Supg003Error>;

    /// Replace the group's matrix. `ADMIN`'s matrix is fixed, and a group cannot drop its own
    /// READ / WRITE on `SUPG003` when the actor belongs to it.
    async fn save_matrix(
        &self,
        id: Uuid,
        cmd: SaveMatrixCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PermissionMatrixDto, Supg003Error>;

    /// Delete a group with its matrix. `ADMIN` cannot be deleted and a group that still has
    /// users or user assignments (or is otherwise referenced) is refused as `InUse`.
    async fn delete_group(&self, id: Uuid) -> Result<(), Supg003Error>;
}
