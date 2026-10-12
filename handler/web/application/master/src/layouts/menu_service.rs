//! Menu HTTP Service
//!
//! Fetches the signed-in account's sidebar menu tree from SUPG002 (`/api/v1/su/supg002/menu`),
//! which is built from `db_program` and filtered by the account's READ permissions.

use serde::{Deserialize, Serialize};
use web_package::core::i18n::t_fmt;
use web_package::core::requestor::HttpClient;

/// API path of the menu tree (SUPG002 = `/su/supg002` under `/api/v1`)
pub const SUPG002_MENU_API: &str = "/api/v1/su/supg002/menu";

/// One node of the menu tree: a folder (has children) or a program page
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuNodeDto {
    /// Program code, e.g. `DBPG003`
    pub code: String,
    /// SPA route; `None` for a folder without a page of its own
    pub path: Option<String>,
    pub icon: Option<String>,
    pub label_th: String,
    pub label_en: String,
    pub sort_order: i32,
    pub children: Vec<MenuNodeDto>,
}

impl MenuNodeDto {
    /// Thai label, English when the Thai one is blank
    #[allow(dead_code)]
    pub fn label(&self) -> String {
        self.label_for("th")
    }

    /// Label translated according to current language code ("th" or "en")
    pub fn label_for(&self, lang: &str) -> String {
        if lang.trim().eq_ignore_ascii_case("en") && !self.label_en.is_empty() {
            self.label_en.clone()
        } else if !self.label_th.is_empty() {
            self.label_th.clone()
        } else {
            self.label_en.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct MenuService {
    http: HttpClient,
}

impl Default for MenuService {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuService {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    /// Fetch the menu tree for the current session
    pub async fn fetch_menu_tree(&self) -> Result<Vec<MenuNodeDto>, String> {
        match self.http.get(SUPG002_MENU_API).send().await {
            Ok(res) if res.is_success() => res
                .auto::<Vec<MenuNodeDto>>()
                .map_err(|e| t_fmt("common.menu.parse_failed", "แปลงข้อมูลเมนูล้มเหลว: {error}", &[("error", &e.to_string())])),
            Ok(res) => Err(t_fmt("common.menu.fetch_failed", "ไม่สามารถดึงเมนูได้ (HTTP {status})", &[("status", &res.status().to_string())])),
            Err(e) => Err(t_fmt("common.menu.connect_failed", "ไม่สามารถเชื่อมต่อเซิร์ฟเวอร์ได้: {error}", &[("error", &e.to_string())])),
        }
    }
}
