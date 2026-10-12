//! Update a sub-district's names / status (optimistic lock; code and parents are fixed)

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use super::dbpg005_executor::Dbpg005CommandExecutor;
use super::geo_input_rules::thai_name;
use super::geo_label_writes::put_sub_district_label;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    ENGLISH_LANGUAGE_ID, GeoNodeDto, THAI_LANGUAGE_ID, UpdateGeoNodeCommand, db_sub_district,
};

impl Dbpg005CommandExecutor {
    pub(super) async fn update_sub_district(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error> {
        let th = thai_name(&cmd.name_th)?;
        let existing = db_sub_district::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg005Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg005Error::VersionConflict);
        }
        let next = existing.row_version + 1;

        let txn = self.db.begin().await?;
        let mut am: db_sub_district::ActiveModel = existing.into();
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        am.row_version = Set(next);
        am.update(&txn).await?;
        put_sub_district_label(&txn, id, THAI_LANGUAGE_ID, th, actor_id, next).await?;
        if !cmd.name_en.trim().is_empty() {
            put_sub_district_label(&txn, id, ENGLISH_LANGUAGE_ID, cmd.name_en.trim().to_string(), actor_id, next).await?;
        }
        txn.commit().await?;

        self.queries
            .get_sub_district(id)
            .await?
            .ok_or_else(|| Dbpg005Error::Internal("Failed to retrieve updated sub-district".to_string()))
    }
}
