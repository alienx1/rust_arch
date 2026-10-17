//! DBPG000 query: retrieve KPI summary of organizations and companies

use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::features::db::dbpg000::errors::Dbpg000Error;
use crate::features::db::dbpg000::models::{OrganizationSummaryDto, db_company, db_organization};

use super::db_dbpg000_queries::DbDbpg000Queries;
use crate::features::db::view_scope_filters::by_company;

impl DbDbpg000Queries {
    pub(super) async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        let total_orgs = db_organization::Entity::find().count(&self.db).await?;
        let active_orgs = db_organization::Entity::find()
            .filter(db_organization::Column::Active.eq(true))
            .count(&self.db)
            .await?;

        let total_comps = by_company(db_company::Entity::find(), db_company::Column::Id).count(&self.db).await?;
        let active_comps = by_company(db_company::Entity::find(), db_company::Column::Id)
            .filter(db_company::Column::Active.eq(true))
            .count(&self.db)
            .await?;

        Ok(OrganizationSummaryDto {
            total_organizations: total_orgs,
            active_organizations: active_orgs,
            total_companies: total_comps,
            active_companies: active_comps,
        })
    }
}
