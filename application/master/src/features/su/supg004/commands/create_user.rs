//! SUPG004 create-user action: creates the identity (no password), then the local account, then
//! tries to issue a recovery link.

use application_core::traits::NewIdentity;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{ActiveModelTrait, TransactionTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::{info, warn};
use uuid::Uuid;

use super::identity_rules::{is_valid_email, normalize_username, provider_error};
use crate::features::su::account_security::{PasswordMailKind, PasswordMailResult};
use super::supg004_executor::Supg004CommandExecutor;
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{
    CreateUserCommand, CreateUserResultDto, is_admin_group, su_user,
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
            // No password yet: the account signs in only after setting one through the emailed link
            must_change_password: Set(true),
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

        // The first password is set through a link e-mailed to the new account holder. The account
        // exists either way; when the mail cannot go out the result carries the link instead.
        let password_mail = match &self.account_security {
            Some(security) => match security.send_password_mail(id, PasswordMailKind::Setup).await {
                Ok(result) => result,
                Err(e) => {
                    warn!("Account {username} created but its password mail failed: {e}");
                    PasswordMailResult { sent: false, sent_to: None, mail_error: Some(e.to_string()), recovery_link: None }
                }
            },
            None => PasswordMailResult {
                sent: false,
                sent_to: None,
                mail_error: Some("password mail is not configured".to_string()),
                recovery_link: None,
            },
        };
        Ok(CreateUserResultDto { user, password_mail })
    }
}
