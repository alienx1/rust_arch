//! Create a district under a province (unique code per province)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::{english_or, thai_name, unit_code};
use super::geo_label_writes::put_district_label;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreateDistrictCommand, ENGLISH_LANGUAGE_ID, GeoNodeDto, THAI_LANGUAGE_ID, db_district,
    db_province,
};

impl Dbpg005CommandExecutor {
    pub(super) async fn create_district(
        &self,
        cmd: CreateDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        let code = unit_code(&cmd.code)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);

        let province = db_province::Entity::find_by_id(cmd.province_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(format!("Province '{}' does not exist", cmd.province_id)))?;
        let duplicate = db_district::Entity::find()
            .filter(db_district::Column::ProvinceId.eq(province.id))
            .filter(db_district::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg005Error::DuplicateCode(format!(
                "District code '{code}' already exists in this province"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new district: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_district::ActiveModel {
            id: Set(id),
            country_id: Set(province.country_id),
            province_id: Set(province.id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_district_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_district_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_district(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve created district".to_string()))
    }
}
