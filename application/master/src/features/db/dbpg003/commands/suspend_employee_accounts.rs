//! DBPG003 side effect of switching an employee off: the user accounts linked to it are suspended
//! (the identity is disabled and its sessions are revoked, then the local flag is cleared), so
//! nobody keeps signing in as a person who no longer works there. The counterpart is
//! `reactivate_employee_accounts`.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::warn;
use uuid::Uuid;

use super::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use domain::entity::su_user;

impl Dbpg003CommandExecutor {
    /// Suspends every still-active account of `employee_id`. Idempotent (running it for an
    /// employee that is already off just finds nothing left to do), so saving the employee again
    /// retries accounts that could not be suspended the first time.
    pub(super) async fn suspend_employee_accounts(&self, employee_id: Uuid) -> Result<(), Dbpg003Error> {
        let Some(accounts) = &self.accounts else { return Ok(()) };
        let users = su_user::Entity::find()
            .filter(su_user::Column::EmployeeId.eq(employee_id))
            .filter(su_user::Column::Active.eq(true))
            .all(&self.db)
            .await?;

        let mut failed = Vec::new();
        for user in users {
            if let Err(e) = accounts.suspend_user(&user.id.to_string()).await {
                warn!("Could not suspend account {} of switched-off employee {employee_id}: {e}", user.username);
                failed.push(user.username);
            }
        }
        if failed.is_empty() {
            Ok(())
        } else {
            Err(Dbpg003Error::Internal(format!(
                "The employee was switched off but these user accounts could not be suspended: {}. Save again to retry.",
                failed.join(", ")
            )))
        }
    }
}
