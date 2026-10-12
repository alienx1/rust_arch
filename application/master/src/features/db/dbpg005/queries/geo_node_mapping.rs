//! Maps province / district / sub-district / postal code rows to DTOs, loading names and parents
//! only for the rows given

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::HashMap;
use uuid::Uuid;

use super::db_dbpg005_queries::DbDbpg005Queries;
use super::label_lookup::{distinct, district_names, display_name, province_names, sub_district_names};
use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    GeoNodeDto, LEVEL_DISTRICT, LEVEL_PROVINCE, LEVEL_SUB_DISTRICT, PostalCodeDto, db_district,
    db_postal_code, db_province, db_sub_district,
};

impl DbDbpg005Queries {
    pub(super) async fn province_nodes(
        &self,
        rows: Vec<db_province::Model>,
    ) -> Result<Vec<GeoNodeDto>, Dbpg005Error> {
        let names = province_names(&self.db, rows.iter().map(|r| r.id).collect()).await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let (th, en) = names.get(&r.id).cloned().unwrap_or_default();
                GeoNodeDto {
                    id: r.id,
                    level: LEVEL_PROVINCE.to_string(),
                    code: r.code,
                    active: r.active,
                    name_th: th,
                    name_en: en,
                    province_id: None,
                    province_name: String::new(),
                    district_id: None,
                    district_name: String::new(),
                    postal_codes: Vec::new(),
                    row_version: r.row_version,
                }
            })
            .collect())
    }

    pub(super) async fn district_nodes(
        &self,
        rows: Vec<db_district::Model>,
    ) -> Result<Vec<GeoNodeDto>, Dbpg005Error> {
        let names = district_names(&self.db, rows.iter().map(|r| r.id).collect()).await?;
        let provinces = province_names(&self.db, distinct(rows.iter().map(|r| r.province_id))).await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let (th, en) = names.get(&r.id).cloned().unwrap_or_default();
                GeoNodeDto {
                    id: r.id,
                    level: LEVEL_DISTRICT.to_string(),
                    code: r.code,
                    active: r.active,
                    name_th: th,
                    name_en: en,
                    province_id: Some(r.province_id),
                    province_name: display_name(&provinces, &r.province_id),
                    district_id: None,
                    district_name: String::new(),
                    postal_codes: Vec::new(),
                    row_version: r.row_version,
                }
            })
            .collect())
    }

    pub(super) async fn sub_district_nodes(
        &self,
        rows: Vec<db_sub_district::Model>,
    ) -> Result<Vec<GeoNodeDto>, Dbpg005Error> {
        let ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
        let names = sub_district_names(&self.db, ids.clone()).await?;
        let districts = district_names(&self.db, distinct(rows.iter().map(|r| r.district_id))).await?;
        let provinces = province_names(&self.db, distinct(rows.iter().map(|r| r.province_id))).await?;

        let mut postal: HashMap<Uuid, Vec<String>> = HashMap::new();
        if !ids.is_empty() {
            for p in db_postal_code::Entity::find()
                .filter(db_postal_code::Column::SubDistrictId.is_in(ids))
                .order_by_asc(db_postal_code::Column::PostalCode)
                .all(&self.db)
                .await?
            {
                if let Some(sub) = p.sub_district_id {
                    postal.entry(sub).or_default().push(p.postal_code);
                }
            }
        }

        Ok(rows
            .into_iter()
            .map(|r| {
                let (th, en) = names.get(&r.id).cloned().unwrap_or_default();
                GeoNodeDto {
                    id: r.id,
                    level: LEVEL_SUB_DISTRICT.to_string(),
                    code: r.code,
                    active: r.active,
                    name_th: th,
                    name_en: en,
                    province_id: Some(r.province_id),
                    province_name: display_name(&provinces, &r.province_id),
                    district_id: Some(r.district_id),
                    district_name: display_name(&districts, &r.district_id),
                    postal_codes: postal.remove(&r.id).unwrap_or_default(),
                    row_version: r.row_version,
                }
            })
            .collect())
    }

    pub(super) async fn postal_dtos(
        &self,
        rows: Vec<db_postal_code::Model>,
    ) -> Result<Vec<PostalCodeDto>, Dbpg005Error> {
        let provinces = province_names(&self.db, distinct(rows.iter().map(|r| r.province_id))).await?;
        let districts = district_names(&self.db, distinct(rows.iter().filter_map(|r| r.district_id))).await?;
        let subs = sub_district_names(&self.db, distinct(rows.iter().filter_map(|r| r.sub_district_id))).await?;
        Ok(rows
            .into_iter()
            .map(|r| PostalCodeDto {
                id: r.id,
                postal_code: r.postal_code,
                active: r.active,
                province_name: display_name(&provinces, &r.province_id),
                province_id: r.province_id,
                district_name: r.district_id.map(|d| display_name(&districts, &d)).unwrap_or_default(),
                district_id: r.district_id,
                sub_district_name: r.sub_district_id.map(|s| display_name(&subs, &s)).unwrap_or_default(),
                sub_district_id: r.sub_district_id,
                row_version: r.row_version,
            })
            .collect())
    }
}
