//! Province / district / sub-district name lookups shared by the geography queries

use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::collections::HashSet;
use uuid::Uuid;

use crate::features::db::dbpg002::queries::db_dbpg002_queries::{Names, fold_names};
use crate::features::localization::request_language::pick;
use crate::features::db::dbpg005::models::{db_district_label, db_province_label, db_sub_district_label};

/// The row's name in the request's language (other language when empty)
pub(super) fn display_name(names: &Names, id: &Uuid) -> String {
    names
        .get(id)
        .map(|(th, en)| pick(th, en).to_string())
        .unwrap_or_default()
}

pub(super) async fn province_names(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Names, DbErr> {
    if ids.is_empty() {
        return Ok(Names::new());
    }
    Ok(fold_names(
        db_province_label::Entity::find()
            .filter(db_province_label::Column::ProvinceId.is_in(ids))
            .all(db)
            .await?
            .into_iter()
            .map(|l| (l.province_id, l.language_id, l.label)),
    ))
}

pub(super) async fn district_names(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Names, DbErr> {
    if ids.is_empty() {
        return Ok(Names::new());
    }
    Ok(fold_names(
        db_district_label::Entity::find()
            .filter(db_district_label::Column::DistrictId.is_in(ids))
            .all(db)
            .await?
            .into_iter()
            .map(|l| (l.district_id, l.language_id, l.label)),
    ))
}

pub(super) async fn sub_district_names(db: &DatabaseConnection, ids: Vec<Uuid>) -> Result<Names, DbErr> {
    if ids.is_empty() {
        return Ok(Names::new());
    }
    Ok(fold_names(
        db_sub_district_label::Entity::find()
            .filter(db_sub_district_label::Column::SubDistrictId.is_in(ids))
            .all(db)
            .await?
            .into_iter()
            .map(|l| (l.sub_district_id, l.language_id, l.label)),
    ))
}

pub(super) fn distinct<T: Copy + Eq + std::hash::Hash>(values: impl Iterator<Item = T>) -> Vec<T> {
    values.collect::<HashSet<_>>().into_iter().collect()
}
