//! DBPG005 Commands trait (reads + writes consumed by routers) and executor

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::dbpg005::errors::Dbpg005Error;
use crate::features::db::dbpg005::models::{
    CreateDistrictCommand, CreatePostalCodeCommand, CreateProvinceCommand,
    CreateSubDistrictCommand, Dbpg005SummaryDto, GeoFilterQuery, GeoNodeDto, GeoPageDto,
    PostalCodeDto, PostalFilterQuery, PostalPageDto, UpdateGeoNodeCommand, UpdatePostalCodeCommand,
};

pub mod dbpg005_executor;

mod create_district;
mod create_postal_code;
mod create_province;
mod create_sub_district;
mod delete_district;
mod delete_postal_code;
mod delete_province;
mod delete_sub_district;
mod geo_input_rules;
mod geo_label_writes;
mod get_geo_node;
mod get_postal_code;
mod get_summary;
mod list_geo_nodes;
mod list_postal_codes;
mod update_district;
mod update_postal_code;
mod update_province;
mod update_sub_district;

pub use dbpg005_executor::Dbpg005CommandExecutor;
pub use geo_input_rules::normalize_postal_code;

#[async_trait]
pub trait Dbpg005Commands: Send + Sync {
    async fn list_provinces(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_sub_districts(&self, filter: GeoFilterQuery) -> Result<GeoPageDto, Dbpg005Error>;
    async fn list_postal_codes(
        &self,
        filter: PostalFilterQuery,
    ) -> Result<PostalPageDto, Dbpg005Error>;

    async fn get_province(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_sub_district(&self, id: Uuid) -> Result<Option<GeoNodeDto>, Dbpg005Error>;
    async fn get_postal_code(&self, id: Uuid) -> Result<Option<PostalCodeDto>, Dbpg005Error>;

    async fn get_summary(&self) -> Result<Dbpg005SummaryDto, Dbpg005Error>;

    /// Create a province; its code must be unique within the country
    async fn create_province(
        &self,
        cmd: CreateProvinceCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    /// Create a district; its code must be unique within the province
    async fn create_district(
        &self,
        cmd: CreateDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    /// Create a sub-district; its code must be unique within the district
    async fn create_sub_district(
        &self,
        cmd: CreateSubDistrictCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    /// Update names / status (optimistic lock); code and parents are fixed
    async fn update_province(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    async fn update_district(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    async fn update_sub_district(
        &self,
        id: Uuid,
        cmd: UpdateGeoNodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<GeoNodeDto, Dbpg005Error>;

    /// Create a postal code under the most specific area given
    async fn create_postal_code(
        &self,
        cmd: CreatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error>;

    /// Update a postal code value / status (optimistic lock); its area is fixed
    async fn update_postal_code(
        &self,
        id: Uuid,
        cmd: UpdatePostalCodeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<PostalCodeDto, Dbpg005Error>;

    /// Delete a province with its labels; `InUse` while districts, postal codes or addresses reference it
    async fn delete_province(&self, id: Uuid) -> Result<(), Dbpg005Error>;

    /// Delete a district with its labels; `InUse` while sub-districts, postal codes or addresses reference it
    async fn delete_district(&self, id: Uuid) -> Result<(), Dbpg005Error>;

    /// Delete a sub-district with its labels; `InUse` while postal codes or addresses reference it
    async fn delete_sub_district(&self, id: Uuid) -> Result<(), Dbpg005Error>;

    /// Delete a postal code; `InUse` while addresses reference it
    async fn delete_postal_code(&self, id: Uuid) -> Result<(), Dbpg005Error>;
}
