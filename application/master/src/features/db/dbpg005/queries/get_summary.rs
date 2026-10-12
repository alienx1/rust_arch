//! Row counts per level

use sea_orm::{EntityTrait, PaginatorTrait};

use super::db_dbpg005_queries::DbDbpg005Queries;
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    Dbpg005SummaryDto, db_district, db_postal_code, db_province, db_sub_district,
};

impl DbDbpg005Queries {
    pub(super) async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error> {
        // Counts per level in SQL (no rows loaded)
        Ok(Dbpg005SummaryDto {
            provinces: db_province::Entity::find().count(&self.db).await?,
            districts: db_district::Entity::find().count(&self.db).await?,
            sub_districts: db_sub_district::Entity::find().count(&self.db).await?,
            postal_codes: db_postal_code::Entity::find().count(&self.db).await?,
        })
    }
}
