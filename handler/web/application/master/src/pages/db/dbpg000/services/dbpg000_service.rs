//! DBPG000 HTTP Service
//!
//! Calls the read endpoints of program `DBPG000` under `/api/v1/db/dbpg000`.

use crate::pages::db::dbpg000::models::{
    CompanyDto, CompanyTypeOptionDto, OrganizationDto, OrganizationSummaryDto,
};
use serde::de::DeserializeOwned;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::HttpClient;

/// API base path of program `DBPG000` (= `db_program.path`, lower-cased, under `/api/v1`).
pub const DBPG000_API: &str = "/api/v1/db/dbpg000";

#[derive(Clone, Debug)]
pub struct Dbpg000Service {
    http: HttpClient,
}

impl Default for Dbpg000Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg000Service {
    pub fn new() -> Self {
        Self {
            http: HttpClient::default(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{DBPG000_API}{path}")).send().await {
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
            Err(e) => Err(t_fmt(
                "dbpg000.error.connect_db",
                "ไม่สามารถเชื่อมต่อเซิร์ฟเวอร์ฐานข้อมูลได้: {error}",
                &[("error", &e.to_string())],
            )),
        }
    }

    /// Fetch all organizations
    pub async fn fetch_organizations(&self) -> Result<Vec<OrganizationDto>, String> {
        self.get("/organizations", &t_or("dbpg000.what.org", "องค์กร")).await
    }

    /// Fetch companies with optional filters
    pub async fn fetch_companies(
        &self,
        org_id: Option<String>,
        search: Option<String>,
        active: Option<bool>,
    ) -> Result<Vec<CompanyDto>, String> {
        let mut query = Vec::new();
        if let Some(ref o) = org_id {
            if o != "ALL" && !o.is_empty() {
                query.push(format!("organization_id={o}"));
            }
        }
        if let Some(ref s) = search {
            if !s.trim().is_empty() {
                query.push(format!("search={}", s.trim()));
            }
        }
        if let Some(a) = active {
            query.push(format!("active={a}"));
        }

        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };

        self.get(&format!("/companies{query_str}"), &t_or("dbpg000.what.company", "บริษัท")).await
    }

    /// Fetch company types master lookup
    pub async fn fetch_company_types(&self) -> Result<Vec<CompanyTypeOptionDto>, String> {
        self.get("/companies/types", &t_or("dbpg000.what.company_type", "ประเภทบริษัท")).await
    }

    /// Fetch organization summary KPIs
    pub async fn fetch_summary(&self) -> Result<OrganizationSummaryDto, String> {
        self.get("/organizations/summary", &t_or("dbpg000.what.org_summary", "สรุปองค์กร")).await
    }
}
