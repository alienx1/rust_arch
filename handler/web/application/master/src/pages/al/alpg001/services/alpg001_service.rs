//! ALPG001 HTTP service: the process list and the reprocess action of
//! `/api/v1/al/alpg001` (served by api-master).

use web_package::core::errors::HttpError;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::HttpClient;

use crate::api_client::query_string;
use crate::pages::al::alpg000::services::alpg000_service::day_start;
use crate::pages::al::alpg001::models::{ProcessFilters, ProcessPageDto};
use crate::routes::percent_encode;

/// API base path of program `ALPG001`
pub const ALPG001_API: &str = "/api/v1/al/alpg001";

#[derive(Clone, Debug, PartialEq)]
pub enum Alpg001Error {
    /// 403: no right to reprocess
    Forbidden,
    /// 409: the process is running / already queued
    Conflict(String),
    /// 503: the queue (Redpanda) does not answer
    Unavailable(String),
    Failed(String),
}

impl Alpg001Error {
    pub fn message(&self) -> String {
        match self {
            Self::Forbidden => t_or("alpg001.error.forbidden", "ไม่มีสิทธิ์ดำเนินการ (ต้องมีสิทธิ์แก้ไขโปรแกรม ALPG001)"),
            Self::Conflict(_) => t_or(
                "alpg001.error.conflict",
                "กระบวนการนี้กำลังทำงานหรืออยู่ในคิวแล้ว ไม่สามารถประมวลผลซ้ำได้ในตอนนี้",
            ),
            Self::Unavailable(_) => t_or("alpg001.error.unavailable", "ระบบคิว (Redpanda) ไม่พร้อมใช้งาน กรุณาลองใหม่ภายหลัง"),
            Self::Failed(m) => m.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Alpg001Service {
    http: HttpClient,
}

impl Default for Alpg001Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Alpg001Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    /// `GET /processes?...`
    pub async fn fetch_processes(
        &self,
        filters: &ProcessFilters,
        page: u64,
        limit: u64,
    ) -> Result<ProcessPageDto, Alpg001Error> {
        let enc = |v: &str| Some(percent_encode(v.trim()));
        let query = query_string(vec![
            ("status", enc(&filters.status)),
            ("user", enc(&filters.user)),
            ("from", day_start(&filters.date_from, 0).map(|v| percent_encode(&v))),
            ("to", day_start(&filters.date_to, 1).map(|v| percent_encode(&v))),
            ("page", Some(page.max(1).to_string())),
            ("limit", (limit > 0).then(|| limit.to_string())),
        ]);
        let what = t_or("alpg001.what.processes", "รายการกระบวนการ");
        let failed = |status: String| {
            Alpg001Error::Failed(t_fmt(
                "common.error.fetch_failed",
                "ไม่สามารถดึงข้อมูล{what}ได้ (HTTP {status})",
                &[("what", &what), ("status", &status)],
            ))
        };
        match self.http.get(&format!("{ALPG001_API}/processes{query}")).send().await {
            Ok(res) => res.auto::<ProcessPageDto>().map_err(|e| {
                Alpg001Error::Failed(t_fmt(
                    "common.error.parse",
                    "แปลงข้อมูล{what}ล้มเหลว: {error}",
                    &[("what", &what), ("error", &e.to_string())],
                ))
            }),
            Err(HttpError::ApiError { status, .. }) => Err(failed(status.to_string())),
            Err(HttpError::Forbidden) => Err(failed("403".to_string())),
            Err(HttpError::Unauthorized) => Err(failed("401".to_string())),
            Err(e) => Err(Alpg001Error::Failed(format!(
                "{}: {e}",
                t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
            ))),
        }
    }

    /// `POST /processes/{id}/reprocess`: queues the process again (202)
    pub async fn reprocess(&self, id: &str) -> Result<(), Alpg001Error> {
        let path = format!("{ALPG001_API}/processes/{}/reprocess", percent_encode(id));
        match self.http.post(&path).send().await {
            Ok(_) => Ok(()),
            Err(HttpError::Forbidden) => Err(Alpg001Error::Forbidden),
            Err(HttpError::ApiError { status: 409, message }) => Err(Alpg001Error::Conflict(message)),
            Err(HttpError::ApiError { status: 503, message }) => Err(Alpg001Error::Unavailable(message)),
            Err(e) => Err(Alpg001Error::Failed(e.to_string())),
        }
    }
}
