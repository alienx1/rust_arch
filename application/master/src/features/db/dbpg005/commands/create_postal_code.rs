//! Create a postal code under the most specific area given

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::info;
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::normalize_postal_code;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreatePostalCodeCommand, PostalCodeDto, db_district, db_postal_code, db_province,
    db_sub_district,
};

impl Dbpg005CommandExecutor {
    pub(super) async fn create_postal_code(
        &self,
        cmd: CreatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error> {
        let postal_code = normalize_postal_code(&cmd.postal_code)?;

        // The most specific area decides the levels above it
        let (country_id, province_id, district_id, sub_district_id) =
            if let Some(sub_id) = cmd.sub_district_id {
                let sub = db_sub_district::Entity::find_by_id(sub_id)
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| Dbpg005Error::NotFound(format!("Sub-district '{sub_id}' does not exist")))?;
                (sub.country_id, sub.province_id, Some(sub.district_id), Some(sub.id))
            } else if let Some(district_id) = cmd.district_id {
                let d = db_district::Entity::find_by_id(district_id)
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| Dbpg005Error::NotFound(format!("District '{district_id}' does not exist")))?;
                (d.country_id, d.province_id, Some(d.id), None)
            } else {
                let p = db_province::Entity::find_by_id(cmd.province_id)
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| Dbpg005Error::NotFound(format!("Province '{}' does not exist", cmd.province_id)))?;
                (p.country_id, p.id, None, None)
            };

        let mut dup = db_postal_code::Entity::find()
            .filter(db_postal_code::Column::PostalCode.eq(&postal_code))
            .filter(db_postal_code::Column::ProvinceId.eq(province_id));
        dup = match sub_district_id {
            Some(s) => dup.filter(db_postal_code::Column::SubDistrictId.eq(s)),
            None => dup.filter(db_postal_code::Column::SubDistrictId.is_null()),
        };
        if dup.one(&self.db).await?.is_some() {
            return Err(Dbpg005Error::DuplicateCode(format!(
                "Postal code '{postal_code}' already exists for this area"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new postal code: id={id}, code={postal_code}");
        db_postal_code::ActiveModel {
            id: Set(id),
            country_id: Set(country_id),
            province_id: Set(province_id),
            district_id: Set(district_id),
            sub_district_id: Set(sub_district_id),
            postal_code: Set(postal_code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&self.db)
        .await?;

        self.queries
            .get_postal_code(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve created postal code".to_string()))
    }
}
