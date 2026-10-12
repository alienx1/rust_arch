//! `SeaORM` Entity for `su_permission_group`: a named set of permissions (the matrix of SUPG003).
//! Where it applies is decided per user by `su_user_assignment`, not by the group.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(schema_name = "arch", table_name = "su_permission_group")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub code: String,
    pub active: bool,
    pub change_by: Option<Uuid>,
    pub row_version: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::su_permission::Entity")]
    SuPermission,
    #[sea_orm(has_many = "super::su_user::Entity")]
    SuUser,
    #[sea_orm(has_many = "super::su_user_assignment::Entity")]
    SuUserAssignment,
}

impl Related<super::su_permission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SuPermission.def()
    }
}

impl Related<super::su_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SuUser.def()
    }
}

impl Related<super::su_user_assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SuUserAssignment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
