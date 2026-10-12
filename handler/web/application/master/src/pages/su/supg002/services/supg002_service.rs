//! SUPG002 HTTP Service
//!
//! Calls the program-management endpoints of `SUPG002` under `/api/v1/su/supg002`.

use web_package::core::i18n::t_or;

use crate::api_client::{ApiClient, query_string};
use crate::pages::su::supg002::models::{
    CreateProgramInput, ProgramDto, ProgramOptionsDto, ProgramPageDto, UpdateProgramInput,
};

/// API base path of program `SUPG002` (= `db_program.path` under `/api/v1`).
pub const SUPG002_API: &str = "/api/v1/su/supg002";

/// Query for the program list; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct ProgramListQuery {
    pub system_id: Option<String>,
    pub module_id: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: u64,
    pub limit: u64,
}

impl ProgramListQuery {
    fn to_query_string(&self) -> String {
        query_string(vec![
            ("system_id", self.system_id.clone()),
            ("module_id", self.module_id.clone()),
            ("search", self.search.clone()),
            ("active", self.active.map(|a| a.to_string())),
            ("page", Some(self.page.max(1).to_string())),
            ("limit", (self.limit > 0).then(|| self.limit.to_string())),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct Supg002Service {
    api: ApiClient,
}

impl Default for Supg002Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg002Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(SUPG002_API) }
    }

    pub async fn fetch_options(&self) -> Result<ProgramOptionsDto, String> {
        self.api.get("/options", &t_or("supg002.what.options", "ตัวเลือก")).await
    }

    pub async fn fetch_programs(&self, q: &ProgramListQuery) -> Result<ProgramPageDto, String> {
        self.api.get(&format!("/programs{}", q.to_query_string()), &t_or("supg002.what.program", "โปรแกรม")).await
    }

    pub async fn fetch_program(&self, id: &str) -> Result<ProgramDto, String> {
        self.api.get(&format!("/programs/{id}"), &t_or("supg002.what.program", "โปรแกรม")).await
    }

    pub async fn create_program(&self, i: CreateProgramInput) -> Result<ProgramDto, String> {
        self.api.post_cbor("/programs", &i, &t_or("supg002.what.program", "โปรแกรม")).await
    }

    pub async fn update_program(&self, id: &str, i: UpdateProgramInput) -> Result<ProgramDto, String> {
        self.api.put_cbor(&format!("/programs/{id}"), &i, &t_or("supg002.what.program", "โปรแกรม")).await
    }

    pub async fn delete_program(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/programs/{id}"), &t_or("supg002.what.program", "โปรแกรม")).await
    }
}
