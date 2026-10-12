//! DBPG003 person name label writes shared by employee create and update

use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, Set};
use uuid::Uuid;

use super::employee_input_rules::{optional, required};
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID, db_person_name_label,
};

/// Name fields shared by the create and update commands
pub(super) struct NameInput<'a> {
    pub(super) first_th: &'a str,
    pub(super) middle_th: &'a str,
    pub(super) last_th: &'a str,
    pub(super) nickname_th: &'a str,
    pub(super) first_en: &'a str,
    pub(super) middle_en: &'a str,
    pub(super) last_en: &'a str,
    pub(super) nickname_en: &'a str,
}

/// Insert or update one language's person name label.
#[allow(clippy::too_many_arguments)]
async fn upsert_name_label<C: ConnectionTrait>(
    conn: &C,
    person_id: Uuid,
    language_id: Uuid,
    first: String,
    middle: &str,
    last: String,
    nickname: &str,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), DbErr> {
    match db_person_name_label::Entity::find_by_id((person_id, language_id))
        .one(conn)
        .await?
    {
        Some(existing) => {
            let mut am: db_person_name_label::ActiveModel = existing.into();
            am.first_name = Set(first);
            am.middle_name = Set(optional(middle));
            am.last_name = Set(last);
            am.nickname = Set(optional(nickname));
            am.change_by = Set(actor_id);
            am.row_version = Set(version);
            am.update(conn).await?;
        }
        None => {
            db_person_name_label::ActiveModel {
                person_name_id: Set(person_id),
                language_id: Set(language_id),
                first_name: Set(first),
                middle_name: Set(optional(middle)),
                last_name: Set(last),
                nickname: Set(optional(nickname)),
                change_by: Set(actor_id),
                row_version: Set(1),
            }
            .insert(conn)
            .await?;
        }
    }
    Ok(())
}

/// Writes the Thai label (required) and the English label (copied from Thai on create when the
/// English name is empty; left untouched on update when it is empty).
pub(super) async fn write_names<C: ConnectionTrait>(
    conn: &C,
    person_id: Uuid,
    n: &NameInput<'_>,
    is_create: bool,
    actor_id: Option<Uuid>,
    version: i64,
) -> Result<(), Dbpg003Error> {
    let first_th = required(n.first_th, "Thai first name")?;
    let last_th = required(n.last_th, "Thai last name")?;
    upsert_name_label(
        conn,
        person_id,
        THAI_LANGUAGE_ID,
        first_th.clone(),
        n.middle_th,
        last_th.clone(),
        n.nickname_th,
        actor_id,
        version,
    )
    .await?;

    let en_given = !n.first_en.trim().is_empty() && !n.last_en.trim().is_empty();
    if en_given {
        upsert_name_label(
            conn,
            person_id,
            ENGLISH_LANGUAGE_ID,
            n.first_en.trim().to_string(),
            n.middle_en,
            n.last_en.trim().to_string(),
            n.nickname_en,
            actor_id,
            version,
        )
        .await?;
    } else if is_create {
        upsert_name_label(
            conn,
            person_id,
            ENGLISH_LANGUAGE_ID,
            first_th,
            n.middle_th,
            last_th,
            n.nickname_th,
            actor_id,
            version,
        )
        .await?;
    }
    Ok(())
}
