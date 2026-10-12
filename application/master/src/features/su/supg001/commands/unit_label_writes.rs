//! SUPG001 label upserts shared by the system and module writes

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::features::su::supg001::models::{db_module_label, db_system_label};

pub(super) async fn put_system_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_system_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_system_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_system_label::ActiveModel {
                system_id: Set(id),
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

pub(super) async fn put_module_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_module_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_module_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_module_label::ActiveModel {
                module_id: Set(id),
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
