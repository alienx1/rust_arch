//! `SeaORM` Entity for `nt_notification` (a notification to one user)

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(schema_name = "arch", table_name = "nt_notification")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub recipient: String,
    pub kind: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: Json,
    pub link: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub read_at: Option<DateTimeWithTimeZone>,
    pub row_version: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
