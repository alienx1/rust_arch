//! DBPG001 HTTP Service
//!
//! Calls the endpoints of program `DBPG001` under `/api/v1/db/dbpg001` (form lookups + writes).

use crate::pages::db::dbpg001::models::{
    CompanyDto, CompanyTypeOptionDto, CreateCompanyInput, CreateOrgInput, OrganizationDto,
    UpdateCompanyInput, UpdateOrgInput,
};
use crate::api_client::ApiClient;
use serde::{Serialize, de::DeserializeOwned};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::HttpClient;

/// API base path of program `DBPG001` (= `db_program.path`, lower-cased, under `/api/v1`).
pub const DBPG001_API: &str = "/api/v1/db/dbpg001";

fn connect_err() -> String {
    t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
}

#[derive(Clone, Debug)]
pub struct Dbpg001Service {
    http: HttpClient,
}

impl Default for Dbpg001Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg001Service {
    pub fn new() -> Self {
        Self {
            http: HttpClient::default(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{DBPG001_API}{path}")).send().await {
            Ok(res) if res.is_success() => res
                .auto::<T>()
                .map_err(|e| {
                    t_fmt(
                        "common.error.parse",
                        "แปลงข้อมูล{what}ล้มเหลว: {error}",
                        &[("what", what), ("error", &e.to_string())],
                    )
                }),
            Ok(res) => Err(t_fmt(
                "common.error.fetch_failed",
                "ไม่สามารถดึงข้อมูล{what}ได้ (HTTP {status})",
                &[("what", what), ("status", &res.status().to_string())],
            )),
            Err(e) => Err(format!("{}: {e}", connect_err())),
        }
    }

    async fn post_cbor<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        what: &str,
    ) -> Result<T, String> {
        let req = self
            .http
            .post(&format!("{DBPG001_API}{path}"))
            .cbor(body)
            .map_err(|e| format!("{}: {e}", connect_err()))?;
        Self::finish(req.send().await, what, true)
    }

    async fn put_cbor<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        what: &str,
    ) -> Result<T, String> {
        let req = self
            .http
            .put(&format!("{DBPG001_API}{path}"))
            .cbor(body)
            .map_err(|e| format!("{}: {e}", connect_err()))?;
        Self::finish(req.send().await, what, false)
    }

    fn finish<T: DeserializeOwned, E: std::fmt::Display>(
        res: Result<web_package::core::requestor::HttpResponse, E>,
        what: &str,
        creating: bool,
    ) -> Result<T, String> {
        match res {
            Ok(res) if res.is_success() => res.auto::<T>().map_err(|e| {
                let (key, fallback) = if creating {
                    ("common.error.parse_create", "แปลงข้อมูลการสร้าง{what}ล้มเหลว: {error}")
                } else {
                    ("common.error.parse_update", "แปลงข้อมูลการแก้ไข{what}ล้มเหลว: {error}")
                };
                t_fmt(key, fallback, &[("what", what), ("error", &e.to_string())])
            }),
            Ok(res) => {
                let (key, fallback) = if creating {
                    ("common.error.create_failed", "สร้าง{what}ไม่สำเร็จ: {error}")
                } else {
                    ("common.error.update_failed", "แก้ไข{what}ไม่สำเร็จ: {error}")
                };
                Err(t_fmt(key, fallback, &[("what", what), ("error", &res.text())]))
            }
            Err(e) => Err(format!("{}: {e}", connect_err())),
        }
    }

    /// `PUT <path>` with the raw image file (`image/*`), or `DELETE <path>` when `file` is `None`;
    /// answers the updated record
    async fn image_call<T: DeserializeOwned>(
        &self,
        path: &str,
        file: Option<(Vec<u8>, String)>,
        what: &str,
    ) -> Result<T, String> {
        let url = format!("{DBPG001_API}{path}");
        let req = match file {
            Some((bytes, content_type)) => self.http.put(&url).bytes(bytes, &content_type),
            None => self.http.delete(&url),
        };
        Self::finish(req.send().await, what, false)
    }

    pub async fn set_organization_image(&self, id: &str, bytes: Vec<u8>, content_type: String) -> Result<OrganizationDto, String> {
        self.image_call(&format!("/organizations/{id}/image"), Some((bytes, content_type)), &t_or("common.what.image", "รูปภาพ")).await
    }

    pub async fn remove_organization_image(&self, id: &str) -> Result<OrganizationDto, String> {
        self.image_call(&format!("/organizations/{id}/image"), None, &t_or("common.what.image", "รูปภาพ")).await
    }

    pub async fn set_company_image(&self, id: &str, bytes: Vec<u8>, content_type: String) -> Result<CompanyDto, String> {
        self.image_call(&format!("/companies/{id}/image"), Some((bytes, content_type)), &t_or("common.what.image", "รูปภาพ")).await
    }

    pub async fn remove_company_image(&self, id: &str) -> Result<CompanyDto, String> {
        self.image_call(&format!("/companies/{id}/image"), None, &t_or("common.what.image", "รูปภาพ")).await
    }

    /// Organizations for the parent selector
    pub async fn fetch_organizations(&self) -> Result<Vec<OrganizationDto>, String> {
        self.get("/organizations", &t_or("dbpg001.what.org", "องค์กร")).await
    }

    /// Organization to edit
    pub async fn fetch_organization(&self, id: &str) -> Result<OrganizationDto, String> {
        self.get(&format!("/organizations/{id}"), &t_or("dbpg001.what.org", "องค์กร")).await
    }

    /// Company to edit
    pub async fn fetch_company(&self, id: &str) -> Result<CompanyDto, String> {
        self.get(&format!("/companies/{id}"), &t_or("dbpg001.what.company", "บริษัท")).await
    }

    /// Company types for the type selector
    pub async fn fetch_company_types(&self) -> Result<Vec<CompanyTypeOptionDto>, String> {
        self.get("/companies/types", &t_or("dbpg001.what.company_type", "ประเภทบริษัท")).await
    }

    pub async fn create_organization(&self, input: CreateOrgInput) -> Result<OrganizationDto, String> {
        self.post_cbor("/organizations", &input, &t_or("dbpg001.what.org", "องค์กร")).await
    }

    pub async fn update_organization(
        &self,
        id: &str,
        input: UpdateOrgInput,
    ) -> Result<OrganizationDto, String> {
        self.put_cbor(&format!("/organizations/{id}"), &input, &t_or("dbpg001.what.org", "องค์กร")).await
    }

    pub async fn create_company(&self, input: CreateCompanyInput) -> Result<CompanyDto, String> {
        self.post_cbor("/companies", &input, &t_or("dbpg001.what.company", "บริษัท")).await
    }

    pub async fn update_company(
        &self,
        id: &str,
        input: UpdateCompanyInput,
    ) -> Result<CompanyDto, String> {
        self.put_cbor(&format!("/companies/{id}"), &input, &t_or("dbpg001.what.company", "บริษัท")).await
    }

    /// Deletes an organization; fails with the server's message when it is still referenced
    pub async fn delete_organization(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG001_API).delete(&format!("/organizations/{id}"), &t_or("dbpg001.what.org", "องค์กร")).await
    }

    /// Deletes a company; fails with the server's message when it is still referenced
    pub async fn delete_company(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG001_API).delete(&format!("/companies/{id}"), &t_or("dbpg001.what.company", "บริษัท")).await
    }
}

