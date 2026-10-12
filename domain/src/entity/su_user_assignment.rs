//! `SeaORM` Entity for `su_user_assignment` (a permission group a user holds at one scope)

use super::sea_orm_active_enums::ScopeLevel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(schema_name = "arch", table_name = "su_user_assignment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission_group_id: Uuid,
    pub scope_level: ScopeLevel,
    pub organization_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub change_by: Option<Uuid>,
    pub row_version: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::su_user::Entity",
        from = "Column::UserId",
        to = "super::su_user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SuUser,
    #[sea_orm(
        belongs_to = "super::su_permission_group::Entity",
        from = "Column::PermissionGroupId",
        to = "super::su_permission_group::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SuPermissionGroup,
}

impl Related<super::su_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SuUser.def()
    }
}

impl Related<super::su_permission_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SuPermissionGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
