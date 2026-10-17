//! DBPG003 HTTP Service
//!
//! Calls the endpoints of program `DBPG003` under `/api/v1/db/dbpg003`.

use crate::pages::db::dbpg003::models::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeInput, Dbpg003SummaryDto,
    DepartmentOptionDto, EmployeeDto, EmployeePageDto, PrefixOptionDto, UpdateEmployeeInput,
};
use crate::api_client::ApiClient;
use serde::{Serialize, de::DeserializeOwned};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::{HttpClient, HttpResponse};

/// API base path of program `DBPG003` (= `db_program.path` under `/api/v1`).
pub const DBPG003_API: &str = "/api/v1/db/dbpg003";

fn connect_err() -> String {
    t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
}

/// Query for the employee list; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct EmployeeListQuery {
    pub company_id: Option<String>,
    pub branch_id: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: u64,
    pub limit: u64,
}

impl EmployeeListQuery {
    fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        for (key, value) in [
            ("company_id", &self.company_id),
            ("branch_id", &self.branch_id),
            ("search", &self.search),
        ] {
            if let Some(v) = value.as_deref().map(str::trim) {
                if !v.is_empty() && v != "ALL" {
                    parts.push(format!("{key}={v}"));
                }
            }
        }
        if let Some(a) = self.active {
            parts.push(format!("active={a}"));
        }
        parts.push(format!("page={}", self.page.max(1)));
        if self.limit > 0 {
            parts.push(format!("limit={}", self.limit));
        }
        format!("?{}", parts.join("&"))
    }
}

#[derive(Clone, Debug)]
pub struct Dbpg003Service {
    http: HttpClient,
}

impl Default for Dbpg003Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg003Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{DBPG003_API}{path}")).send().await {
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
            .post(&format!("{DBPG003_API}{path}"))
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
            .put(&format!("{DBPG003_API}{path}"))
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

    pub async fn fetch_summary(&self) -> Result<Dbpg003SummaryDto, String> {
        self.get("/summary", &t_or("dbpg003.what.summary", "สรุปพนักงาน")).await
    }

    pub async fn fetch_company_options(&self) -> Result<Vec<CompanyOptionDto>, String> {
        self.get("/companies", &t_or("dbpg003.what.company", "บริษัท")).await
    }

    pub async fn fetch_branch_options(&self) -> Result<Vec<BranchOptionDto>, String> {
        self.get("/branches", &t_or("dbpg003.what.branch", "สาขา")).await
    }

    pub async fn fetch_department_options(&self) -> Result<Vec<DepartmentOptionDto>, String> {
        self.get("/departments", &t_or("dbpg003.what.department", "แผนก")).await
    }

    pub async fn fetch_prefixes(&self) -> Result<Vec<PrefixOptionDto>, String> {
        self.get("/prefixes", &t_or("dbpg003.what.prefix", "คำนำหน้าชื่อ")).await
    }

    pub async fn fetch_employees(&self, query: &EmployeeListQuery) -> Result<EmployeePageDto, String> {
        self.get(&format!("/employees{}", query.to_query_string()), &t_or("dbpg003.what.employee", "พนักงาน")).await
    }

    pub async fn fetch_employee(&self, id: &str) -> Result<EmployeeDto, String> {
        self.get(&format!("/employees/{id}"), &t_or("dbpg003.what.employee", "พนักงาน")).await
    }

    pub async fn create_employee(&self, input: CreateEmployeeInput) -> Result<EmployeeDto, String> {
        self.post_cbor("/employees", &input, &t_or("dbpg003.what.employee", "พนักงาน")).await
    }

    pub async fn update_employee(
        &self,
        id: &str,
        input: UpdateEmployeeInput,
    ) -> Result<EmployeeDto, String> {
        self.put_cbor(&format!("/employees/{id}"), &input, &t_or("dbpg003.what.employee", "พนักงาน")).await
    }

    /// Hard delete; the server answers 409 (with its explanation) while the row is still referenced
    /// `PUT` the raw image file (`image/*`), or `DELETE` the photo when `file` is `None`; answers the
    /// updated employee
    pub async fn set_employee_image(&self, id: &str, file: Option<(Vec<u8>, String)>) -> Result<EmployeeDto, String> {
        let url = format!("{DBPG003_API}/employees/{id}/image");
        let req = match file {
            Some((bytes, content_type)) => self.http.put(&url).bytes(bytes, &content_type),
            None => self.http.delete(&url),
        };
        Self::finish(req.send().await, &t_or("common.what.image", "รูปภาพ"), false)
    }

    pub async fn delete_employee(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG003_API).delete(&format!("/employees/{id}"), &t_or("dbpg003.what.employee", "พนักงาน")).await
    }
}
