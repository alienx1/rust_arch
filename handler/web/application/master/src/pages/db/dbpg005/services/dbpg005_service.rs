//! DBPG005 HTTP Service
//!
//! Calls the endpoints of program `DBPG005` under `/api/v1/db/dbpg005`.

use crate::api_client::{ApiClient, query_string};
use web_package::core::i18n::t_or;
use crate::pages::db::dbpg005::models::{
    CreateDistrictInput, CreatePostalCodeInput, CreateProvinceInput, CreateSubDistrictInput,
    Dbpg005SummaryDto, GeoNodeDto, GeoPageDto, PostalCodeDto, PostalPageDto, UpdateGeoNodeInput,
    UpdatePostalCodeInput,
};

/// API base path of program `DBPG005` (= `db_program.path` under `/api/v1`).
pub const DBPG005_API: &str = "/api/v1/db/dbpg005";

/// Query for the geography lists; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct GeoListQuery {
    pub province_id: Option<String>,
    pub district_id: Option<String>,
    pub sub_district_id: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: u64,
    pub limit: u64,
}

impl GeoListQuery {
    fn to_query_string(&self) -> String {
        query_string(vec![
            ("province_id", self.province_id.clone()),
            ("district_id", self.district_id.clone()),
            ("sub_district_id", self.sub_district_id.clone()),
            ("search", self.search.clone()),
            ("active", self.active.map(|a| a.to_string())),
            ("page", Some(self.page.max(1).to_string())),
            ("limit", (self.limit > 0).then(|| self.limit.to_string())),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct Dbpg005Service {
    api: ApiClient,
}

impl Default for Dbpg005Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg005Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(DBPG005_API) }
    }

    pub async fn fetch_summary(&self) -> Result<Dbpg005SummaryDto, String> {
        self.api.get("/summary", &t_or("dbpg005.what.summary", "สรุปข้อมูลภูมิศาสตร์")).await
    }

    pub async fn fetch_provinces(&self, q: &GeoListQuery) -> Result<GeoPageDto, String> {
        self.api.get(&format!("/provinces{}", q.to_query_string()), &t_or("dbpg005.what.province", "จังหวัด")).await
    }

    pub async fn fetch_districts(&self, q: &GeoListQuery) -> Result<GeoPageDto, String> {
        self.api.get(&format!("/districts{}", q.to_query_string()), &t_or("dbpg005.what.district", "อำเภอ")).await
    }

    pub async fn fetch_sub_districts(&self, q: &GeoListQuery) -> Result<GeoPageDto, String> {
        self.api.get(&format!("/sub-districts{}", q.to_query_string()), &t_or("dbpg005.what.sub_district", "ตำบล")).await
    }

    pub async fn fetch_postal_codes(&self, q: &GeoListQuery) -> Result<PostalPageDto, String> {
        self.api.get(&format!("/postal-codes{}", q.to_query_string()), &t_or("dbpg005.what.postal_code", "รหัสไปรษณีย์")).await
    }

    pub async fn fetch_province(&self, id: &str) -> Result<GeoNodeDto, String> {
        self.api.get(&format!("/provinces/{id}"), &t_or("dbpg005.what.province", "จังหวัด")).await
    }

    pub async fn fetch_district(&self, id: &str) -> Result<GeoNodeDto, String> {
        self.api.get(&format!("/districts/{id}"), &t_or("dbpg005.what.district", "อำเภอ")).await
    }

    pub async fn fetch_sub_district(&self, id: &str) -> Result<GeoNodeDto, String> {
        self.api.get(&format!("/sub-districts/{id}"), &t_or("dbpg005.what.sub_district", "ตำบล")).await
    }

    pub async fn fetch_postal_code(&self, id: &str) -> Result<PostalCodeDto, String> {
        self.api.get(&format!("/postal-codes/{id}"), &t_or("dbpg005.what.postal_code", "รหัสไปรษณีย์")).await
    }

    pub async fn create_province(&self, i: CreateProvinceInput) -> Result<GeoNodeDto, String> {
        self.api.post_cbor("/provinces", &i, &t_or("dbpg005.what.province", "จังหวัด")).await
    }

    pub async fn create_district(&self, i: CreateDistrictInput) -> Result<GeoNodeDto, String> {
        self.api.post_cbor("/districts", &i, &t_or("dbpg005.what.district", "อำเภอ")).await
    }

    pub async fn create_sub_district(&self, i: CreateSubDistrictInput) -> Result<GeoNodeDto, String> {
        self.api.post_cbor("/sub-districts", &i, &t_or("dbpg005.what.sub_district", "ตำบล")).await
    }

    pub async fn create_postal_code(&self, i: CreatePostalCodeInput) -> Result<PostalCodeDto, String> {
        self.api.post_cbor("/postal-codes", &i, &t_or("dbpg005.what.postal_code", "รหัสไปรษณีย์")).await
    }

    pub async fn update_province(&self, id: &str, i: UpdateGeoNodeInput) -> Result<GeoNodeDto, String> {
        self.api.put_cbor(&format!("/provinces/{id}"), &i, &t_or("dbpg005.what.province", "จังหวัด")).await
    }

    pub async fn update_district(&self, id: &str, i: UpdateGeoNodeInput) -> Result<GeoNodeDto, String> {
        self.api.put_cbor(&format!("/districts/{id}"), &i, &t_or("dbpg005.what.district", "อำเภอ")).await
    }

    pub async fn update_sub_district(&self, id: &str, i: UpdateGeoNodeInput) -> Result<GeoNodeDto, String> {
        self.api.put_cbor(&format!("/sub-districts/{id}"), &i, &t_or("dbpg005.what.sub_district", "ตำบล")).await
    }

    pub async fn update_postal_code(&self, id: &str, i: UpdatePostalCodeInput) -> Result<PostalCodeDto, String> {
        self.api.put_cbor(&format!("/postal-codes/{id}"), &i, &t_or("dbpg005.what.postal_code", "รหัสไปรษณีย์")).await
    }

    pub async fn delete_province(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/provinces/{id}"), &t_or("dbpg005.what.province", "จังหวัด")).await
    }

    pub async fn delete_district(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/districts/{id}"), &t_or("dbpg005.what.district", "อำเภอ")).await
    }

    pub async fn delete_sub_district(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/sub-districts/{id}"), &t_or("dbpg005.what.sub_district", "ตำบล")).await
    }

    pub async fn delete_postal_code(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/postal-codes/{id}"), &t_or("dbpg005.what.postal_code", "รหัสไปรษณีย์")).await
    }
}
