//! SUPG002 parent-folder check shared by program create and update

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::db_program;

impl Supg002CommandExecutor {
    /// The parent folder, which must exist and be in the same system
    pub(super) async fn check_parent(
        &self,
        parent_id: Option<Uuid>,
        system_id: Uuid,
        self_id: Option<Uuid>,
    ) -> Result<(), Supg002Error> {
        let Some(parent_id) = parent_id else { return Ok(()) };
        if self_id == Some(parent_id) {
            return Err(Supg002Error::Validation("A program cannot be its own parent".to_string()));
        }
        let mut current = db_program::Entity::find_by_id(parent_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg002Error::NotFound(format!("Parent program '{parent_id}' does not exist")))?;
        if current.system_id != system_id {
            return Err(Supg002Error::Validation(
                "The parent folder must be in the same system".to_string(),
            ));
        }
        // Walk up: reaching the program being edited means the move would create a cycle
        if let Some(me) = self_id {
            let mut guard = 0;
            while let Some(next) = current.program_id {
                if next == me {
                    return Err(Supg002Error::Validation(
                        "A program cannot be moved below one of its own descendants".to_string(),
                    ));
                }
                guard += 1;
                if guard > 64 {
                    return Err(Supg002Error::Internal("Program hierarchy is too deep".to_string()));
                }
                current = match db_program::Entity::find_by_id(next).one(&self.db).await? {
                    Some(p) => p,
                    None => break,
                };
            }
        }
        Ok(())
    }
}
