//! DBPG004 HTTP Service
//!
//! Calls the endpoints of program `DBPG004` under `/api/v1/db/dbpg004`.

use crate::pages::db::dbpg004::models::{
    CreateLookupInput, Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupPageDto,
    UpdateLookupInput,
};
use crate::api_client::ApiClient;
use serde::{Serialize, de::DeserializeOwned};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::{HttpClient, HttpResponse};

/// API base path of program `DBPG004` (= `db_program.path` under `/api/v1`).
pub const DBPG004_API: &str = "/api/v1/db/dbpg004";

fn connect_err() -> String {
    t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
}

/// Query for the lookup list; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct LookupListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: u64,
    pub limit: u64,
}

impl LookupListQuery {
    fn to_query_string(&self) -> String {
        let mut parts = Vec::new();
        for (key, value) in [("category", &self.category), ("search", &self.search)] {
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
pub struct Dbpg004Service {
    http: HttpClient,
}

impl Default for Dbpg004Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg004Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{DBPG004_API}{path}")).send().await {
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
            .post(&format!("{DBPG004_API}{path}"))
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
            .put(&format!("{DBPG004_API}{path}"))
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
            Ok(res) if res.is_success() => res
                .auto::<T>()
                .map_err(|e| {
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

    pub async fn fetch_summary(&self) -> Result<Dbpg004SummaryDto, String> {
        self.get("/summary", &t_or("dbpg004.what.summary", "สรุปข้อมูลอ้างอิง")).await
    }

    pub async fn fetch_categories(&self) -> Result<Vec<LookupCategoryDto>, String> {
        self.get("/categories", &t_or("dbpg004.what.category", "หมวดหมู่")).await
    }

    pub async fn fetch_lookups(&self, query: &LookupListQuery) -> Result<LookupPageDto, String> {
        self.get(&format!("/lookups{}", query.to_query_string()), &t_or("dbpg004.what.lookup", "ข้อมูลอ้างอิง")).await
    }

    pub async fn fetch_lookup(&self, id: &str) -> Result<LookupDto, String> {
        self.get(&format!("/lookups/{id}"), &t_or("dbpg004.what.lookup", "ข้อมูลอ้างอิง")).await
    }

    pub async fn create_lookup(&self, input: CreateLookupInput) -> Result<LookupDto, String> {
        self.post_cbor("/lookups", &input, &t_or("dbpg004.what.lookup", "ข้อมูลอ้างอิง")).await
    }

    pub async fn update_lookup(&self, id: &str, input: UpdateLookupInput) -> Result<LookupDto, String> {
        self.put_cbor(&format!("/lookups/{id}"), &input, &t_or("dbpg004.what.lookup", "ข้อมูลอ้างอิง")).await
    }

    /// Hard delete; the server answers 409 (with its explanation) while the row is still referenced
    pub async fn delete_lookup(&self, id: &str) -> Result<(), String> {
        ApiClient::new(DBPG004_API).delete(&format!("/lookups/{id}"), &t_or("dbpg004.what.lookup_value", "ค่าอ้างอิง")).await
    }
}
