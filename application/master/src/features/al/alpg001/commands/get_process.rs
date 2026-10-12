//! ALPG001 get-one action

use chrono::Utc;
use uuid::Uuid;

use super::alpg001_executor::Alpg001CommandExecutor;
use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::ProcessDto;

impl Alpg001CommandExecutor {
    pub(super) async fn get_process_action(&self, id: &str) -> Result<Option<ProcessDto>, Alpg001Error> {
        let id = Uuid::parse_str(id.trim()).map_err(|_| Alpg001Error::Validation("Process id must be a UUID".into()))?;
        Ok(self.queries.find(id).await?.map(|m| ProcessDto::from_model(m, Utc::now())))
    }
}
