//! Login gate: user, employee, organization and company must all be active

use domain::entity::{db_company, db_employee, db_organization, su_user};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::queries::DbSupg005Queries;

impl DbSupg005Queries {
    pub(super) async fn is_login_allowed(&self, identifier: &str) -> Result<bool, Supg005Error> {
        let username = identifier.split('@').next().unwrap_or(identifier);
        let user = su_user::Entity::find()
            .filter(su_user::Column::Username.eq(username))
            .one(&self.db)
            .await?;
        let Some(user) = user else { return Ok(true) };
        if !user.active {
            return Ok(false);
        }
        // An inactive organization blocks everyone in it; an inactive company only its own users.
        let Some(employee) = db_employee::Entity::find_by_id(user.employee_id).one(&self.db).await?
        else {
            return Ok(true);
        };
        // An employee who is switched off cannot sign in, whatever the account flag says
        if !employee.active {
            return Ok(false);
        }
        let org_active = db_organization::Entity::find_by_id(employee.organization_id)
            .one(&self.db)
            .await?
            .is_none_or(|o| o.active);
        let company_active = db_company::Entity::find_by_id(employee.company_id)
            .one(&self.db)
            .await?
            .is_none_or(|c| c.active);
        Ok(org_active && company_active)
    }
}
