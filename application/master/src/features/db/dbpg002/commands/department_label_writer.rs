//! DBPG002 department label upsert shared by department create and update

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::features::db::dbpg002::models::db_department_label;

pub(super) async fn upsert_department_label<C: ConnectionTrait>(
    conn: &C,
    department_id: Uuid,
    language_id: Uuid,
    label: String,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_department_label::Entity::find_by_id((department_id, language_id))
        .one(conn)
        .await?
    {
        Some(existing) => {
            let mut am: db_department_label::ActiveModel = existing.into();
            am.label = Set(label);
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_department_label::ActiveModel {
                department_id: Set(department_id),
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
