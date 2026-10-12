//! ALPG000 HTTP Service
//!
//! Calls the endpoints of program `ALPG000` under `/api/v1/al/alpg000`. Uses the HTTP client
//! directly (not `ApiClient`) because a 503 from the list means "audit service down".

use serde::de::DeserializeOwned;
use web_package::core::errors::HttpError;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::HttpClient;

use crate::api_client::query_string;
use crate::pages::al::alpg000::models::{
    AuditFilters, AuditLogDto, AuditLogPageDto, AuditStatusDto, ProgramOptionDto,
};
use crate::routes::percent_encode;

/// API base path of program `ALPG000`
pub const ALPG000_API: &str = "/api/v1/al/alpg000";

/// Why a call failed
#[derive(Clone, Debug, PartialEq)]
pub enum Alpg000Error {
    /// 503: the audit store (Quickwit) is down; carries the server's message
    Unavailable(String),
    /// 404 of a detail call
    NotFound,
    Failed(String),
}

impl Alpg000Error {
    pub fn message(&self) -> String {
        match self {
            Self::Unavailable(m) | Self::Failed(m) => m.clone(),
            Self::NotFound => t_or("alpg000.error.not_found", "ไม่พบเหตุการณ์ที่ต้องการ"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Alpg000Service {
    http: HttpClient,
}

impl Default for Alpg000Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Alpg000Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, Alpg000Error> {
        let failed = |status: String| {
            Alpg000Error::Failed(t_fmt(
                "common.error.fetch_failed",
                "ไม่สามารถดึงข้อมูล{what}ได้ (HTTP {status})",
                &[("what", what), ("status", &status)],
            ))
        };
        match self.http.get(&format!("{ALPG000_API}{path}")).send().await {
            Ok(res) => res.auto::<T>().map_err(|e| {
                Alpg000Error::Failed(t_fmt(
                    "common.error.parse",
                    "แปลงข้อมูล{what}ล้มเหลว: {error}",
                    &[("what", what), ("error", &e.to_string())],
                ))
            }),
            Err(HttpError::ApiError { status: 503, message }) => Err(Alpg000Error::Unavailable(message)),
            Err(HttpError::ApiError { status, .. }) => Err(failed(status.to_string())),
            Err(HttpError::NotFound(_)) => Err(Alpg000Error::NotFound),
            Err(HttpError::Forbidden) => Err(failed("403".to_string())),
            Err(HttpError::Unauthorized) => Err(failed("401".to_string())),
            Err(e) => Err(Alpg000Error::Failed(format!(
                "{}: {e}",
                t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้")
            ))),
        }
    }

    /// `GET /programs`
    pub async fn fetch_programs(&self) -> Result<Vec<ProgramOptionDto>, Alpg000Error> {
        self.get("/programs", &t_or("alpg000.what.programs", "รายการโปรแกรม")).await
    }

    /// `GET /status`
    pub async fn fetch_status(&self) -> Result<AuditStatusDto, Alpg000Error> {
        self.get("/status", &t_or("alpg000.what.status", "สถานะบริการบันทึก")).await
    }

    /// `GET /logs?...`; the time window is turned into `from` now (the end is open)
    pub async fn fetch_logs(
        &self,
        filters: &AuditFilters,
        page: u64,
        limit: u64,
    ) -> Result<AuditLogPageDto, Alpg000Error> {
        let query = filter_query(filters, vec![
            ("page", Some(page.max(1).to_string())),
            ("limit", (limit > 0).then(|| limit.to_string())),
        ]);
        self.get(&format!("/logs{query}"), &t_or("alpg000.what.logs", "บันทึกเหตุการณ์")).await
    }

    /// `POST /exports?...`: queues a CSV of everything `filters` match. The report worker builds
    /// it in the background and pushes a notification; returns the job id. The time window ends
    /// open, like the list.
    pub async fn request_export(&self, filters: &AuditFilters) -> Result<String, Alpg000Error> {
        #[derive(serde::Deserialize)]
        struct Queued {
            job_id: String,
        }
        let path = format!("{ALPG000_API}/exports{}", filter_query(filters, Vec::new()));
        match self.http.post(&path).send().await {
            Ok(res) => res.auto::<Queued>().map(|q| q.job_id).map_err(|e| Alpg000Error::Failed(e.to_string())),
            Err(HttpError::ApiError { status: 503, message }) => Err(Alpg000Error::Unavailable(message)),
            Err(HttpError::Forbidden) => Err(Alpg000Error::Failed(t_or(
                "alpg000.export.forbidden",
                "ไม่มีสิทธิ์ส่งออกรายงาน",
            ))),
            Err(e) => Err(Alpg000Error::Failed(e.to_string())),
        }
    }

    /// `GET /logs/{id}`
    pub async fn fetch_log(&self, id: &str) -> Result<AuditLogDto, Alpg000Error> {
        self.get(
            &format!("/logs/{}", percent_encode(id)),
            &t_or("alpg000.what.log", "รายละเอียดเหตุการณ์"),
        )
        .await
    }
}

/// Query string of the list filters (+ `extra`), shared by the list and the CSV export
fn filter_query(filters: &AuditFilters, extra: Vec<(&str, Option<String>)>) -> String {
    let (from, to) = (day_start(&filters.date_from, 0), day_start(&filters.date_to, 1));
    let enc = |v: &str| Some(percent_encode(v.trim()));
    let mut params = vec![
        ("from", from.map(|v| percent_encode(&v))),
        ("to", to.map(|v| percent_encode(&v))),
        ("kind", enc(&filters.kind)),
        ("user", enc(&filters.user)),
        ("program_code", enc(&filters.program_code.to_uppercase())),
        ("method", enc(&filters.method)),
        ("status_class", enc(&filters.status_class)),
        ("path", enc(&filters.path)),
        ("text", enc(&filters.text)),
    ];
    params.extend(extra);
    query_string(params)
}

/// RFC 3339 instant of local midnight of `date` (`YYYY-MM-DD`, browser time zone) plus
/// `plus_days` days; `None` for an empty or malformed date. `to` = day after the last day, as the
/// API's `to` is exclusive.
pub(crate) fn day_start(date: &str, plus_days: u32) -> Option<String> {
    let d = chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()?;
    use chrono::Datelike;
    let local_ms = js_sys::Date::new_with_year_month_day(d.year() as u32, d.month0() as i32, d.day() as i32 + plus_days as i32).get_time();
    chrono::DateTime::from_timestamp_millis(local_ms as i64)
        .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}
