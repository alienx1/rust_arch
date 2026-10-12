//! SUPG002 localized program names shared by the program DTOs and the form options

use sea_orm::{DatabaseConnection, EntityTrait};

use crate::features::db::dbpg002::queries::db_dbpg002_queries::{Names, fold_names};
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::db_program_label;

pub(super) async fn program_names(db: &DatabaseConnection) -> Result<Names, Supg002Error> {
    Ok(fold_names(
        db_program_label::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|l| (l.program_id, l.language_id, l.label)),
    ))
}
