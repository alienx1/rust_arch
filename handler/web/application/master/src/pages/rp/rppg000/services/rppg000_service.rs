//! RPPG000 HTTP service: the report jobs of the signed-in user (`/api/v1/reports`, served by
//! api-report) and the server-sent-event stream of push notifications.

use serde::de::DeserializeOwned;
use web_package::core::errors::HttpError;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::HttpClient;

use crate::pages::rp::rppg000::models::{ReportJobDto, ReportPreviewDto};
use crate::routes::percent_encode;

/// API base path of the report service
pub const REPORTS_API: &str = "/api/v1/reports";

#[derive(Clone, Debug, PartialEq)]
pub enum Rppg000Error {
    /// 404: no such report (or somebody else's)
    NotFound,
    /// 409: the report has no file (yet)
    NotReady(String),
    Failed(String),
}

impl Rppg000Error {
    pub fn message(&self) -> String {
        match self {
            Self::NotFound => t_or("rppg000.error.not_found", "ไม่พบรายงานที่ต้องการ"),
            Self::NotReady(m) | Self::Failed(m) => m.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rppg000Service {
    http: HttpClient,
}

impl Default for Rppg000Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Rppg000Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, Rppg000Error> {
        let failed = |status: String| {
            Rppg000Error::Failed(t_fmt(
                "common.error.fetch_failed",
                "ไม่สามารถดึงข้อมูล{what}ได้ (HTTP {status})",
                &[("what", what), ("status", &status)],
            ))
        };
        match self.http.get(&format!("{REPORTS_API}{path}")).send().await {
            Ok(res) => res.auto::<T>().map_err(|e| {
                Rppg000Error::Failed(t_fmt(
                    "common.error.parse",
                    "แปลงข้อมูล{what}ล้มเหลว: {error}",
                    &[("what", what), ("error", &e.to_string())],
                ))
            }),
            Err(HttpError::NotFound(_)) => Err(Rppg000Error::NotFound),
            Err(HttpError::ApiError { status: 409, message }) => Err(Rppg000Error::NotReady(message)),
            Err(HttpError::ApiError { status: 410, .. }) => Err(Rppg000Error::Failed(t_or(
                "rppg000.error.file_gone",
                "ไฟล์รายงานนี้ไม่มีอยู่แล้ว (ถูกลบหรือที่เก็บไฟล์ถูกล้าง) กรุณาขอรายงานใหม่",
            ))),
            Err(HttpError::ApiError { status, .. }) => Err(failed(status.to_string())),
            Err(HttpError::Forbidden) => Err(failed("403".to_string())),
            Err(HttpError::Unauthorized) => Err(failed("401".to_string())),
            Err(e) => Err(Rppg000Error::Failed(format!(
                "{}: {e}",
                t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
            ))),
        }
    }

    /// `GET /jobs`
    pub async fn fetch_jobs(&self) -> Result<Vec<ReportJobDto>, Rppg000Error> {
        self.get("/jobs", &t_or("rppg000.what.jobs", "รายการรายงาน")).await
    }

    /// `GET /jobs/{id}/preview?rows=`
    pub async fn fetch_preview(&self, id: &str, rows: usize) -> Result<ReportPreviewDto, Rppg000Error> {
        self.get(
            &format!("/jobs/{}/preview?rows={rows}", percent_encode(id)),
            &t_or("rppg000.what.preview", "ตัวอย่างไฟล์"),
        )
        .await
    }

    /// `GET /jobs/{id}/download`: short-lived signed storage link that saves the file. The
    /// browser opens it itself (a proxy must not follow it: the signature covers the host).
    pub async fn fetch_download_url(&self, id: &str) -> Result<String, Rppg000Error> {
        #[derive(serde::Deserialize)]
        struct Link {
            url: String,
        }
        let link: Link = self
            .get(&format!("/jobs/{}/download", percent_encode(id)), &t_or("rppg000.what.download", "ลิงก์ดาวน์โหลด"))
            .await?;
        Ok(link.url)
    }
}
