//! Shared HTTP helper for the program pages: JSON/CBOR calls to one program's API base path
//! with uniform Thai error messages.

use serde::{Serialize, de::DeserializeOwned};
use web_package::core::errors::HttpError;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::core::requestor::{HttpClient, HttpResponse};

fn connect_error(detail: impl std::fmt::Display) -> String {
    format!("{}: {detail}", t_or("common.error.connect", "ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้"))
}

/// Client for one program API, e.g. `ApiClient::new("/api/v1/db/dbpg005")`
#[derive(Clone, Debug)]
pub struct ApiClient {
    http: HttpClient,
    base: &'static str,
}

impl ApiClient {
    pub fn new(base: &'static str) -> Self {
        Self { http: HttpClient::default(), base }
    }

    /// `GET {base}{path}`; `what` names the data in error messages
    pub async fn get<T: DeserializeOwned>(&self, path: &str, what: &str) -> Result<T, String> {
        match self.http.get(&format!("{}{path}", self.base)).send().await {
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
            Err(e) => Err(connect_error(e)),
        }
    }

    /// `POST {base}{path}` with a CBOR body
    pub async fn post_cbor<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        what: &str,
    ) -> Result<T, String> {
        let req = self
            .http
            .post(&format!("{}{path}", self.base))
            .cbor(body)
            .map_err(connect_error)?;
        finish(req.send().await, what, Verb::Create)
    }

    /// `DELETE {base}{path}`; a 409 answer means the row is still referenced and carries the
    /// server's explanation
    pub async fn delete(&self, path: &str, what: &str) -> Result<(), String> {
        let failed = |error: &str| {
            t_fmt(
                "common.error.delete_failed",
                "ลบ{what}ไม่สำเร็จ: {error}",
                &[("what", what), ("error", error)],
            )
        };
        match self.http.delete(&format!("{}{path}", self.base)).send().await {
            Ok(res) if res.is_success() => Ok(()),
            Ok(res) => Err(failed(&res.text())),
            Err(HttpError::ApiError { message, .. }) => Err(failed(&message)),
            Err(HttpError::Forbidden) => Err(failed(&t_or(
                "common.error.forbidden",
                "ไม่มีสิทธิ์แก้ไขข้อมูล",
            ))),
            Err(HttpError::NotFound(_)) => Err(failed(&t_or(
                "common.error.gone",
                "ไม่พบข้อมูล (อาจถูกลบไปแล้ว)",
            ))),
            Err(e) => Err(connect_error(e)),
        }
    }

    /// `PUT {base}{path}` with a CBOR body
    pub async fn put_cbor<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        what: &str,
    ) -> Result<T, String> {
        let req = self
            .http
            .put(&format!("{}{path}", self.base))
            .cbor(body)
            .map_err(connect_error)?;
        finish(req.send().await, what, Verb::Update)
    }
}

/// What a write was doing, for the messages of a failure
#[derive(Clone, Copy)]
enum Verb {
    Create,
    Update,
}

fn finish<T: DeserializeOwned, E: std::fmt::Display>(
    res: Result<HttpResponse, E>,
    what: &str,
    verb: Verb,
) -> Result<T, String> {
    let (parse_key, parse_fallback, fail_key, fail_fallback) = match verb {
        Verb::Create => (
            "common.error.parse_create",
            "แปลงข้อมูลการสร้าง{what}ล้มเหลว: {error}",
            "common.error.create_failed",
            "สร้าง{what}ไม่สำเร็จ: {error}",
        ),
        Verb::Update => (
            "common.error.parse_update",
            "แปลงข้อมูลการแก้ไข{what}ล้มเหลว: {error}",
            "common.error.update_failed",
            "แก้ไข{what}ไม่สำเร็จ: {error}",
        ),
    };
    match res {
        Ok(res) if res.is_success() => res.auto::<T>().map_err(|e| {
            t_fmt(parse_key, parse_fallback, &[("what", what), ("error", &e.to_string())])
        }),
        Ok(res) => Err(t_fmt(fail_key, fail_fallback, &[("what", what), ("error", &res.text())])),
        Err(e) => Err(connect_error(e)),
    }
}

/// Query string (`?a=b&c=d`) from the set parameters; blank and `ALL` values are left out.
pub fn query_string(params: Vec<(&str, Option<String>)>) -> String {
    let parts: Vec<String> = params
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .filter(|(_, v)| !v.trim().is_empty() && v != "ALL")
        .map(|(k, v)| format!("{k}={}", v.trim()))
        .collect();
    if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
}
