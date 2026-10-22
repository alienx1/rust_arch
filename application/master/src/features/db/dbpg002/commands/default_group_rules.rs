//! DBPG002 rule: a department's default permission group must be an existing, active group

use sea_orm::{ConnectionTrait, EntityTrait};
use uuid::Uuid;

use crate::features::db::dbpg002::errors::Dbpg002Error;
use crate::features::su::supg004::models::is_admin_group;
use domain::entity::su_permission_group;

pub(super) async fn check_default_group(conn: &impl ConnectionTrait, group: Option<Uuid>) -> Result<(), Dbpg002Error> {
    let Some(id) = group else { return Ok(()) };
    match su_permission_group::Entity::find_by_id(id).one(conn).await? {
        None => Err(Dbpg002Error::NotFound(format!("Permission group '{id}' does not exist"))),
        Some(g) if !g.active => {
            Err(Dbpg002Error::Validation(format!("The permission group '{}' is switched off", g.code)))
        }
        // Everyone in the department would become an administrator
        Some(g) if is_admin_group(&g.code) => Err(Dbpg002Error::Validation(
            "The ADMIN group cannot be a department's default permission group".to_string(),
        )),
        Some(_) => Ok(()),
    }
}
