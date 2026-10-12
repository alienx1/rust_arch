//! Create a sub-district under a district (unique code per district)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::{english_or, thai_name, unit_code};
use super::geo_label_writes::put_sub_district_label;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreateSubDistrictCommand, ENGLISH_LANGUAGE_ID, GeoNodeDto, THAI_LANGUAGE_ID, db_district,
    db_sub_district,
};

impl Dbpg005CommandExecutor {
    pub(super) async fn create_sub_district(
        &self,
        cmd: CreateSubDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        let code = unit_code(&cmd.code)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);

        let district = db_district::Entity::find_by_id(cmd.district_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(format!("District '{}' does not exist", cmd.district_id)))?;
        let duplicate = db_sub_district::Entity::find()
            .filter(db_sub_district::Column::DistrictId.eq(district.id))
            .filter(db_sub_district::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg005Error::DuplicateCode(format!(
                "Sub-district code '{code}' already exists in this district"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new sub-district: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_sub_district::ActiveModel {
            id: Set(id),
            country_id: Set(district.country_id),
            province_id: Set(district.province_id),
            district_id: Set(district.id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_sub_district_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_sub_district_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_sub_district(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve created sub-district".to_string()))
    }
}
