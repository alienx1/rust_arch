//! Create a province under the default country (unique code per country)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::{english_or, thai_name, unit_code};
use super::geo_label_writes::put_province_label;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreateProvinceCommand, ENGLISH_LANGUAGE_ID, GeoNodeDto, THAI_LANGUAGE_ID, db_country,
    db_province,
};

impl Dbpg005CommandExecutor {
    /// The country new provinces belong to: the first active country by code
    async fn default_country(&self) -> Result<db_country::Model, Dbpg005Error> {
        db_country::Entity::find()
            .filter(db_country::Column::Active.eq(true))
            .order_by_asc(db_country::Column::Code)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("No active country is configured".to_string()))
    }

    pub(super) async fn create_province(
        &self,
        cmd: CreateProvinceCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        let code = unit_code(&cmd.code)?;
        let th = thai_name(&cmd.name_th)?;
        let en = english_or(&cmd.name_en, &th);
        let country = self.default_country().await?;

        let duplicate = db_province::Entity::find()
            .filter(db_province::Column::CountryId.eq(country.id))
            .filter(db_province::Column::Code.eq(&code))
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg005Error::DuplicateCode(format!("Province code '{code}' already exists")));
        }

        let id = Uuid::now_v7();
        info!("Creating new province: id={id}, code={code}");
        let txn = self.db.begin().await?;
        db_province::ActiveModel {
            id: Set(id),
            country_id: Set(country.id),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        put_province_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, 1).await?;
        put_province_label(&txn, id, ENGLISH_LANGUAGE_ID, en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_province(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve created province".to_string()))
    }
}
