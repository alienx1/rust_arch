//! SUPG002 program label upsert shared by program create and update

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::features::su::supg002::models::db_program_label;

pub(super) async fn put_label<C: ConnectionTrait>(
    conn: &C,
    id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_program_label::Entity::find_by_id((id, language_id)).one(conn).await? {
        Some(existing) => {
            let mut am: db_program_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_program_label::ActiveModel {
                program_id: Set(id),
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
