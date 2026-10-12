//! SUPG004 create-user action: creates the identity (no password), then the local account, then
//! tries to issue a recovery link.

use application_core::traits::NewIdentity;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{ActiveModelTrait, TransactionTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::{info, warn};
use uuid::Uuid;

use super::identity_rules::{is_valid_email, normalize_username, provider_error};
use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    CreateUserCommand, CreateUserResultDto, RecoveryLinkDto, is_admin_group, su_user,
};
use domain::entity::db_employee;

impl Supg004CommandExecutor {
    pub(super) async fn create_user(
        &self,
        cmd: CreateUserCommand,
        actor_id: Option<Uuid>,
        can_assign_admin: bool,
    ) -> Result<CreateUserResultDto, Supg004Error> {
        let username = normalize_username(&cmd.username)?;
        let email = cmd.email.trim().to_string();
        if !is_valid_email(&email) {
            return Err(Supg004Error::Validation("A valid email address is required".to_string()));
        }
        let identity = self.identity()?.clone();

        let employee = db_employee::Entity::find_by_id(cmd.employee_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Supg004Error::NotFound(format!("Employee '{}' does not exist", cmd.employee_id)))?;
        if !employee.active {
            return Err(Supg004Error::Validation("The employee is not active".to_string()));
        }
        if su_user::Entity::find()
            .filter(su_user::Column::EmployeeId.eq(employee.id))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(Supg004Error::Duplicate("This employee already has an account".to_string()));
        }
        // Case-insensitive, matching the lower-case storage
        if su_user::Entity::find()
            .filter(Expr::expr(Func::lower(Expr::col(su_user::Column::Username))).eq(username.clone()))
            .one(&self.db)
            .await?
            .is_some()
        {
            return Err(Supg004Error::Duplicate(format!("Username '{username}' is already taken")));
        }

        let (assignments, groups) = self.validate_assignments(&cmd.assignments).await?;
        // The first assignment's group is the primary group of the account
        let primary_group = assignments[0].0;
        if groups.values().any(|code| is_admin_group(code)) && !can_assign_admin {
            return Err(Supg004Error::NotAllowed(
                "Placing an account in the ADMIN group needs APPROVE on SUPG004".to_string(),
            ));
        }

        let names = self
            .queries
            .employee_names(employee.id)
            .await?
            .ok_or_else(|| Supg004Error::Internal("The employee has no name".to_string()))?;

        // Identity first: it is the part that can fail for reasons outside the database
        let identity_id = identity
            .create_identity(NewIdentity {
                email: email.clone(),
                username: username.clone(),
                first_name: names.first_name,
                last_name: names.last_name,
            })
            .await
            .map_err(provider_error)?;

        let id = Uuid::now_v7();
        info!("Creating user account: id={id}, username={username}");
        let saved = async {
            let txn = self.db.begin().await?;
            su_user::ActiveModel {
            id: Set(id),
            employee_id: Set(employee.id),
            username: Set(username.clone()),
            permission_group_id: Set(primary_group),
            password_hash: Set(String::new()),
            password_type: Set("argon2id".to_string()),
            created_time: Set(chrono::Utc::now().fixed_offset()),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
            .insert(&txn)
            .await?;
            self.write_assignments(&txn, id, &assignments, actor_id).await?;
            txn.commit().await?;
            Ok::<(), Supg004Error>(())
        }
        .await;

        if let Err(e) = saved {
            // Undo the identity so a retry does not hit a conflict on a half-created account
            if let Err(undo) = identity.delete_identity(&identity_id).await {
                warn!("Could not remove identity {identity_id} after a failed insert: {undo}");
            }
            return Err(e);
        }

        let user = self
            .queries
            .get_user(id)
            .await?
            .ok_or_else(|| Supg004Error::Internal("Failed to retrieve created account".to_string()))?;

        let (recovery_link, recovery_error) = match identity.create_recovery_link(&username).await {
            Ok(link) => (Some(RecoveryLinkDto { url: link.url, expires_at: link.expires_at }), None),
            Err(e) => {
                warn!("Account {username} created but no recovery link could be issued: {e}");
                (None, Some(e.to_string()))
            }
        };
        Ok(CreateUserResultDto { user, recovery_link, recovery_error })
    }
}
