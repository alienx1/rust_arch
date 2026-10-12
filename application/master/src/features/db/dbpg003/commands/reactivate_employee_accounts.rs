//! DBPG003 side effect of switching an employee back on: the user accounts linked to it are
//! reactivated (identity enabled again, then the local flag set), mirroring the suspension done
//! when the employee was switched off.
//!
//! Only runs on the off -> on change of the employee, so saving an active employee never undoes a
//! suspension an administrator made on purpose. Accounts that cannot be reactivated are reported
//! and can be switched on by hand in SUPG004 / SUPG005.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tracing::warn;
use uuid::Uuid;

use super::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use domain::entity::su_user;

impl Dbpg003CommandExecutor {
    pub(super) async fn reactivate_employee_accounts(&self, employee_id: Uuid) -> Result<(), Dbpg003Error> {
        let Some(accounts) = &self.accounts else { return Ok(()) };
        let users = su_user::Entity::find()
            .filter(su_user::Column::EmployeeId.eq(employee_id))
            .filter(su_user::Column::Active.eq(false))
            .all(&self.db)
            .await?;

        let mut failed = Vec::new();
        for user in users {
            if let Err(e) = accounts.reactivate_user(&user.id.to_string()).await {
                warn!("Could not reactivate account {} of employee {employee_id}: {e}", user.username);
                failed.push(user.username);
            }
        }
        if failed.is_empty() {
            Ok(())
        } else {
            Err(Dbpg003Error::Internal(format!(
                "The employee was switched on but these user accounts could not be reactivated: {}. Switch them on in SUPG004.",
                failed.join(", ")
            )))
        }
    }
}
