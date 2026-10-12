//! SUPG003 HTTP Service
//!
//! Calls the endpoints of program `SUPG003` under `/api/v1/su/supg003`.

use web_package::core::i18n::t_or;
use crate::api_client::{ApiClient, query_string};
use crate::pages::su::supg003::models::{
    CreateGroupInput, PermissionGroupDto, PermissionMatrixDto, SaveMatrixInput, UpdateGroupInput,
};

/// API base path of program `SUPG003` (= `db_program.path` under `/api/v1`).
pub const SUPG003_API: &str = "/api/v1/su/supg003";

#[derive(Clone, Debug)]
pub struct Supg003Service {
    api: ApiClient,
}

impl Default for Supg003Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg003Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(SUPG003_API) }
    }

    pub async fn fetch_groups(&self) -> Result<Vec<PermissionGroupDto>, String> {
        let q = query_string(vec![]);
        self.api.get(&format!("/groups{q}"), &t_or("supg003.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn fetch_group(&self, id: &str) -> Result<PermissionGroupDto, String> {
        self.api.get(&format!("/groups/{id}"), &t_or("supg003.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn fetch_matrix(&self, id: &str) -> Result<PermissionMatrixDto, String> {
        self.api.get(&format!("/groups/{id}/matrix"), &t_or("supg003.what.matrix", "เมทริกซ์สิทธิ์")).await
    }

    pub async fn create_group(&self, i: CreateGroupInput) -> Result<PermissionGroupDto, String> {
        self.api.post_cbor("/groups", &i, &t_or("supg003.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn update_group(&self, id: &str, i: UpdateGroupInput) -> Result<PermissionGroupDto, String> {
        self.api.put_cbor(&format!("/groups/{id}"), &i, &t_or("supg003.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn delete_group(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/groups/{id}"), &t_or("supg003.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn save_matrix(&self, id: &str, i: SaveMatrixInput) -> Result<PermissionMatrixDto, String> {
        self.api.put_cbor(&format!("/groups/{id}/matrix"), &i, &t_or("supg003.what.matrix", "เมทริกซ์สิทธิ์")).await
    }
}
