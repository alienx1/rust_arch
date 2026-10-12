//! ALPG000 get-one-event action

use super::alpg000_executor::Alpg000CommandExecutor;
use super::search_logs::map_store_error;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::AuditLogDto;

impl Alpg000CommandExecutor {
    pub(super) async fn get_log_action(&self, id: &str) -> Result<Option<AuditLogDto>, Alpg000Error> {
        let id = id.trim();
        if id.is_empty() {
            return Err(Alpg000Error::Validation("Event id is required".to_string()));
        }
        Ok(self.audit.get(id).await.map_err(map_store_error)?.map(Into::into))
    }
}
