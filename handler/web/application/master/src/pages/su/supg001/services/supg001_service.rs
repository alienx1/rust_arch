//! SUPG001 HTTP Service
//!
//! Calls the endpoints of program `SUPG001` under `/api/v1/su/supg001`.

use web_package::core::i18n::t_or;

use crate::api_client::{ApiClient, query_string};
use crate::pages::su::supg001::models::{
    CreateModuleInput, CreateSystemInput, ModuleDto, SystemDto, UpdateUnitInput,
};

/// API base path of program `SUPG001` (= `db_program.path` under `/api/v1`).
pub const SUPG001_API: &str = "/api/v1/su/supg001";

#[derive(Clone, Debug)]
pub struct Supg001Service {
    api: ApiClient,
}

impl Default for Supg001Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg001Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(SUPG001_API) }
    }

    pub async fn fetch_systems(&self) -> Result<Vec<SystemDto>, String> {
        self.api.get("/systems", &t_or("supg001.what.system", "ระบบ")).await
    }

    pub async fn fetch_modules(&self, system_id: Option<String>) -> Result<Vec<ModuleDto>, String> {
        let q = query_string(vec![("system_id", system_id)]);
        self.api.get(&format!("/modules{q}"), &t_or("supg001.what.module", "โมดูล")).await
    }

    pub async fn fetch_system(&self, id: &str) -> Result<SystemDto, String> {
        self.api.get(&format!("/systems/{id}"), &t_or("supg001.what.system", "ระบบ")).await
    }

    pub async fn fetch_module(&self, id: &str) -> Result<ModuleDto, String> {
        self.api.get(&format!("/modules/{id}"), &t_or("supg001.what.module", "โมดูล")).await
    }

    pub async fn create_system(&self, i: CreateSystemInput) -> Result<SystemDto, String> {
        self.api.post_cbor("/systems", &i, &t_or("supg001.what.system", "ระบบ")).await
    }

    pub async fn update_system(&self, id: &str, i: UpdateUnitInput) -> Result<SystemDto, String> {
        self.api.put_cbor(&format!("/systems/{id}"), &i, &t_or("supg001.what.system", "ระบบ")).await
    }

    pub async fn create_module(&self, i: CreateModuleInput) -> Result<ModuleDto, String> {
        self.api.post_cbor("/modules", &i, &t_or("supg001.what.module", "โมดูล")).await
    }

    pub async fn update_module(&self, id: &str, i: UpdateUnitInput) -> Result<ModuleDto, String> {
        self.api.put_cbor(&format!("/modules/{id}"), &i, &t_or("supg001.what.module", "โมดูล")).await
    }

    pub async fn delete_system(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/systems/{id}"), &t_or("supg001.what.system", "ระบบ")).await
    }

    pub async fn delete_module(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/modules/{id}"), &t_or("supg001.what.module", "โมดูล")).await
    }
}
