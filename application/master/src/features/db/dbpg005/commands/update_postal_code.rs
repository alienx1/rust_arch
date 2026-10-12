//! Update a postal code value / status (optimistic lock; its area is fixed)

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::normalize_postal_code;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{PostalCodeDto, UpdatePostalCodeCommand, db_postal_code};

impl Dbpg005CommandExecutor {
    pub(super) async fn update_postal_code(
        &self,
        id: Uuid,
        cmd: UpdatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error> {
        let postal_code = normalize_postal_code(&cmd.postal_code)?;
        let existing = db_postal_code::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg005Error::VersionConflict);
        }

        if existing.postal_code != postal_code {
            let mut dup = db_postal_code::Entity::find()
                .filter(db_postal_code::Column::PostalCode.eq(&postal_code))
                .filter(db_postal_code::Column::ProvinceId.eq(existing.province_id))
                .filter(db_postal_code::Column::Id.ne(id));
            dup = match existing.sub_district_id {
                Some(s) => dup.filter(db_postal_code::Column::SubDistrictId.eq(s)),
                None => dup.filter(db_postal_code::Column::SubDistrictId.is_null()),
            };
            if dup.one(&self.db).await?.is_some() {
                return Err(Dbpg005Error::DuplicateCode(format!(
                    "Postal code '{postal_code}' already exists for this area"
                )));
            }
        }

        let next = existing.row_version + 1;
        let mut am: db_postal_code::ActiveModel = existing.into();
        am.postal_code = Set(postal_code);
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        am.row_version = Set(next);
        am.update(&self.db).await?;

        self.queries
            .get_postal_code(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve updated postal code".to_string()))
    }
}
