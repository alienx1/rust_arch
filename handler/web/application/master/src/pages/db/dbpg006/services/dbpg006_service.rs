//! DBPG006 HTTP Service
//!
//! Calls the endpoints of program `DBPG006` under `/api/v1/db/dbpg006`.

use crate::api_client::{ApiClient, query_string};
use crate::pages::db::dbpg006::models::{
    LabelCategoryDto, LabelEntryDto, LabelPageDto, LanguageOptionDto, SaveLabelInput,
};
use crate::routes::percent_encode;
use web_package::core::i18n::t_or;

/// API base path of program `DBPG006` (= `db_program.path` under `/api/v1`).
pub const DBPG006_API: &str = "/api/v1/db/dbpg006";

/// Query for the label list; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct LabelListQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub page: u64,
    pub limit: u64,
}

impl LabelListQuery {
    fn to_query_string(&self) -> String {
        query_string(vec![
            ("category", self.category.as_deref().map(percent_encode)),
            ("search", self.search.as_deref().map(percent_encode)),
            ("page", Some(self.page.max(1).to_string())),
            ("limit", (self.limit > 0).then(|| self.limit.to_string())),
        ])
    }
}

/// `?category=…&key=…` identifying one entry (both percent-encoded)
fn entry_query(category: &str, key: &str) -> String {
    format!("?category={}&key={}", percent_encode(category), percent_encode(key))
}

#[derive(Clone, Debug)]
pub struct Dbpg006Service {
    api: ApiClient,
}

impl Default for Dbpg006Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Dbpg006Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(DBPG006_API) }
    }

    pub async fn fetch_languages(&self) -> Result<Vec<LanguageOptionDto>, String> {
        self.api.get("/languages", &t_or("dbpg006.what.language", "ภาษา")).await
    }

    pub async fn fetch_categories(&self) -> Result<Vec<LabelCategoryDto>, String> {
        self.api.get("/categories", &t_or("dbpg006.what.category", "หมวดหมู่")).await
    }

    pub async fn fetch_labels(&self, query: &LabelListQuery) -> Result<LabelPageDto, String> {
        self.api
            .get(&format!("/labels{}", query.to_query_string()), &t_or("dbpg006.what.label", "ข้อความแปลภาษา"))
            .await
    }

    pub async fn fetch_label(&self, category: &str, key: &str) -> Result<LabelEntryDto, String> {
        self.api
            .get(&format!("/labels/entry{}", entry_query(category, key)), &t_or("dbpg006.what.label", "ข้อความแปลภาษา"))
            .await
    }

    pub async fn create_label(&self, input: SaveLabelInput) -> Result<LabelEntryDto, String> {
        self.api.post_cbor("/labels", &input, &t_or("dbpg006.what.label", "ข้อความแปลภาษา")).await
    }

    pub async fn update_label(&self, input: SaveLabelInput) -> Result<LabelEntryDto, String> {
        self.api.put_cbor("/labels", &input, &t_or("dbpg006.what.label", "ข้อความแปลภาษา")).await
    }

    /// Hard delete of every language of the entry
    pub async fn delete_label(&self, category: &str, key: &str) -> Result<(), String> {
        self.api
            .delete(&format!("/labels{}", entry_query(category, key)), &t_or("dbpg006.what.label", "ข้อความแปลภาษา"))
            .await
    }
}
