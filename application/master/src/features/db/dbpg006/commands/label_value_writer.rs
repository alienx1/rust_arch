//! DBPG006 label persistence: writes the languages of one entry inside a caller's transaction

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{db_label, db_language};

/// Every language id must belong to an active language
pub(super) async fn ensure_active_languages<C: ConnectionTrait>(
    conn: &C,
    values: &[(Uuid, Option<String>)],
) -> Result<(), Dbpg006Error> {
    let ids: Vec<Uuid> = values.iter().map(|(id, _)| *id).collect();
    let active: Vec<Uuid> = db_language::Entity::find()
        .filter(db_language::Column::Id.is_in(ids.clone()))
        .filter(db_language::Column::Active.eq(true))
        .all(conn)
        .await?
        .into_iter()
        .map(|l| l.id)
        .collect();
    match ids.iter().find(|id| !active.contains(id)) {
        Some(id) => Err(Dbpg006Error::Validation(format!(
            "Language {id} is not an active language"
        ))),
        None => Ok(()),
    }
}

/// Text -> insert (new rows get a v7 id, version 1) or update (version + 1); blank -> delete
pub(super) async fn write_values<C: ConnectionTrait>(
    conn: &C,
    category: &str,
    key: &str,
    values: Vec<(Uuid, Option<String>)>,
    actor_id: Option<Uuid>,
) -> Result<(), Dbpg006Error> {
    for (language_id, text) in values {
        let existing = db_label::Entity::find()
            .filter(db_label::Column::LanguageId.eq(language_id))
            .filter(db_label::Column::Category.eq(category))
            .filter(db_label::Column::Key.eq(key))
            .one(conn)
            .await?;
        match (existing, text) {
            (Some(row), Some(value)) => {
                let version = row.row_version + 1;
                let mut am: db_label::ActiveModel = row.into();
                am.value = Set(value);
                am.change_by = Set(actor_id);
                am.row_version = Set(version);
                am.update(conn).await?;
            }
            (None, Some(value)) => {
                db_label::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    language_id: Set(language_id),
                    category: Set(category.to_string()),
                    key: Set(key.to_string()),
                    value: Set(value),
                    change_by: Set(actor_id),
                    row_version: Set(1),
                }
                .insert(conn)
                .await?;
            }
            (Some(row), None) => {
                db_label::Entity::delete_by_id(row.id).exec(conn).await?;
            }
            (None, None) => {}
        }
    }
    Ok(())
}
