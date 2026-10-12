//! Upserts of province / district / sub-district labels inside a write transaction

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::features::db::dbpg005::models::{
    db_district_label, db_province_label, db_sub_district_label,
};

pub(super) async fn put_province_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_province_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_province_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_province_label::ActiveModel {
                province_id: Set(id),
                language_id: Set(language_id),
                label: Set(label),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(conn)
            .await?;
        }
    }
    Ok(())
}

pub(super) async fn put_district_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_district_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_district_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_district_label::ActiveModel {
                district_id: Set(id),
                language_id: Set(language_id),
                label: Set(label),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(conn)
            .await?;
        }
    }
    Ok(())
}

pub(super) async fn put_sub_district_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_sub_district_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_sub_district_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_sub_district_label::ActiveModel {
                sub_district_id: Set(id),
                language_id: Set(language_id),
                label: Set(label),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(conn)
            .await?;
        }
    }
    Ok(())
}
