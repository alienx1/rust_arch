//! SUPG003 group holders: the distinct users that hold a permission group, either as their primary
//! group (`su_user.permission_group_id`) or through a scope assignment (`su_user_assignment`).

use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use domain::entity::{su_user, su_user_assignment};

/// Distinct users holding `group_id`
pub async fn group_user_ids(db: &impl ConnectionTrait, group_id: Uuid) -> Result<HashSet<Uuid>, DbErr> {
    let mut users: HashSet<Uuid> = su_user::Entity::find()
        .filter(su_user::Column::PermissionGroupId.eq(group_id))
        .all(db)
        .await?
        .into_iter()
        .map(|u| u.id)
        .collect();
    users.extend(
        su_user_assignment::Entity::find()
            .filter(su_user_assignment::Column::PermissionGroupId.eq(group_id))
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.user_id),
    );
    Ok(users)
}

/// Number of distinct users per group, for every group that has any
pub async fn user_counts(db: &impl ConnectionTrait) -> Result<HashMap<Uuid, u64>, DbErr> {
    let mut holders: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
    for u in su_user::Entity::find().all(db).await? {
        holders.entry(u.permission_group_id).or_default().insert(u.id);
    }
    for a in su_user_assignment::Entity::find().all(db).await? {
        holders.entry(a.permission_group_id).or_default().insert(a.user_id);
    }
    Ok(holders.into_iter().map(|(g, s)| (g, s.len() as u64)).collect())
}
