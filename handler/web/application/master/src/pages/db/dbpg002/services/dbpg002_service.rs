//! DBPG002 HTTP Service
//!
//! Calls the endpoints of program `DBPG002` under `/api/v1/db/dbpg002`.

use crate::pages::db::dbpg002::models::{
    BranchDto, BranchTypeOptionDto, PermissionGroupOptionDto, CompanyOptionDto, CreateBranchInput, CreateDepartmentInput,
    Dbpg002SummaryDto, DepartmentDto, UpdateBranchInput, UpdateDepartmentInput,
};
use crate::api_client::ApiClient;
use serde::{Serialize, de::DeserializeOwned};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::{HttpClient, HttpResponse};

/// API base path of program `DBPG002` (= `db_program.path`, lower-cased, under `/api/v1`).
pub const DBPG002_API: &str = "/api/v1/db/dbpg002";

fn connect_err() -> String {
    t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
}

/// Query string (`?a=b&c=d`) from the set parameters; empty when none.
fn query_string(params: Vec<(&str, Option<String>)>) -> String {
    let parts: Vec<String> = params
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .filter(|(_, v)| !v.trim().is_empty() && v != "ALL")
        .map(|(k, v)| format!("{k}={}", v.trim()))
        .collect();
    if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
}

#[derive(Clone, Debug)]
pub struct Dbpg002Service {
    http: HttpClient,
}

impl Default for Dbpg002Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg002Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{DBPG002_API}{path}")).send().await {
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
            .post(&format!("{DBPG002_API}{path}"))
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
            .put(&format!("{DBPG002_API}{path}"))
            .cbor(body)
            .map_err(|e| format!("{}: {e}", connect_err()))?;
        Self::finish(req.send().await, what, false)
    }

    fn finish<T: DeserializeOwned, E: std::fmt::Display>(
        res: Result<HttpResponse, E>,
        what: &str,
        create: bool,
    ) -> Result<T, String> {
        match res {
            Ok(res) if res.is_success() => res.auto::<T>().map_err(|e| {
                let (key, fallback) = if create {
                    ("common.error.parse_create", "แปลงข้อมูลการสร้าง{what}ล้มเหลว: {error}")
                } else {
                    ("common.error.parse_update", "แปลงข้อมูลการแก้ไข{what}ล้มเหลว: {error}")
                };
                t_fmt(key, fallback, &[("what", what), ("error", &e.to_string())])
            }),
            Ok(res) => {
                let (key, fallback) = if create {
                    ("common.error.create_failed", "สร้าง{what}ไม่สำเร็จ: {error}")
                } else {
                    ("common.error.update_failed", "แก้ไข{what}ไม่สำเร็จ: {error}")
                };
                Err(t_fmt(key, fallback, &[("what", what), ("error", &res.text())]))
            }
            Err(e) => Err(format!("{}: {e}", connect_err())),
        }
    }

    pub async fn fetch_summary(&self) -> Result<Dbpg002SummaryDto, String> {
        self.get("/summary", &t_or("dbpg002.what.summary", "สรุปสาขาและแผนก")).await
    }

    pub async fn fetch_company_options(&self) -> Result<Vec<CompanyOptionDto>, String> {
        self.get("/companies", &t_or("dbpg002.what.company", "บริษัท")).await
    }

    pub async fn fetch_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, String> {
        self.get("/options/permission-groups", &t_or("dbpg002.what.permission_group", "กลุ่มสิทธิ์")).await
    }

    pub async fn fetch_branch_types(&self) -> Result<Vec<BranchTypeOptionDto>, String> {
        self.get("/branches/types", &t_or("dbpg002.what.branch_type", "ประเภทสาขา")).await
    }

    pub async fn fetch_branches(
        &self,
        company_id: Option<String>,
        search: Option<String>,
        active: Option<bool>,
    ) -> Result<Vec<BranchDto>, String> {
        let q = query_string(vec![
            ("company_id", company_id),
            ("search", search),
            ("active", active.map(|a| a.to_string())),
        ]);
        self.get(&format!("/branches{q}"), &t_or("dbpg002.what.branch", "สาขา")).await
    }

    pub async fn fetch_branch(&self, id: &str) -> Result<BranchDto, String> {
        self.get(&format!("/branches/{id}"), &t_or("dbpg002.what.branch", "สาขา")).await
    }

    pub async fn fetch_departments(
        &self,
        company_id: Option<String>,
        search: Option<String>,
        active: Option<bool>,
    ) -> Result<Vec<DepartmentDto>, String> {
        let q = query_string(vec![
            ("company_id", company_id),
            ("search", search),
            ("active", active.map(|a| a.to_string())),
        ]);
        self.get(&format!("/departments{q}"), &t_or("dbpg002.what.department", "แผนก")).await
    }

    pub async fn fetch_department(&self, id: &str) -> Result<DepartmentDto, String> {
        self.get(&format!("/departments/{id}"), &t_or("dbpg002.what.department", "แผนก")).await
    }

    pub async fn create_branch(&self, input: CreateBranchInput) -> Result<BranchDto, String> {
        self.post_cbor("/branches", &input, &t_or("dbpg002.what.branch", "สาขา")).await
    }

    pub async fn update_branch(&self, id: &str, input: UpdateBranchInput) -> Result<BranchDto, String> {
        self.put_cbor(&format!("/branches/{id}"), &input, &t_or("dbpg002.what.branch", "สาขา")).await
    }

    pub async fn create_department(&self, input: CreateDepartmentInput) -> Result<DepartmentDto, String> {
        self.post_cbor("/departments", &input, &t_or("dbpg002.what.department", "แผนก")).await
    }

    pub async fn update_department(
        &self,
        id: &str,
        input: UpdateDepartmentInput,
    ) -> Result<DepartmentDto, String> {
        self.put_cbor(&format!("/departments/{id}"), &input, &t_or("dbpg002.what.department", "แผนก")).await
    }

    pub async fn delete_branch(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG002_API).delete(&format!("/branches/{id}"), &t_or("dbpg002.what.branch", "สาขา")).await
    }

    pub async fn delete_department(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG002_API).delete(&format!("/departments/{id}"), &t_or("dbpg002.what.department", "แผนก")).await
    }
}
