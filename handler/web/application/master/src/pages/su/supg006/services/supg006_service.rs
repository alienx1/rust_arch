//! SUPG006 HTTP Service
//!
//! Calls the endpoints of program `SUPG006` under `/api/v1/su/supg006`.

use web_package::core::i18n::t_or;

use crate::api_client::ApiClient;
use crate::pages::su::supg006::models::{ConfigItemDto, SaveConfigInput};

/// API base path of program `SUPG006` (= `db_program.path` under `/api/v1`).
pub const SUPG006_API: &str = "/api/v1/su/supg006";

#[derive(Clone, Debug)]
pub struct Supg006Service {
    api: ApiClient,
}

impl Default for Supg006Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg006Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(SUPG006_API) }
    }

    pub async fn fetch_config(&self) -> Result<Vec<ConfigItemDto>, String> {
        self.api.get("/config", &t_or("supg006.what.config", "การตั้งค่า")).await
    }

    pub async fn save_config(&self, input: SaveConfigInput) -> Result<Vec<ConfigItemDto>, String> {
        self.api.put_cbor("/config", &input, &t_or("supg006.what.config", "การตั้งค่า")).await
    }
}
