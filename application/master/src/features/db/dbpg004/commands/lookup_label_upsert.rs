//! DBPG004 label persistence: insert or update one localized lookup label

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::features::db::dbpg004::models::db_master_lookup_label;

pub(super) async fn upsert_label<C: ConnectionTrait>(
    conn: &C,
    lookup_id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_master_lookup_label::Entity::find_by_id((lookup_id, language_id))
        .one(conn)
        .await?
    {
        Some(existing) => {
            let mut am: db_master_lookup_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_master_lookup_label::ActiveModel {
                master_lookup_id: Set(lookup_id),
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
