//! ALPG000 store status action

use super::alpg000_executor::Alpg000CommandExecutor;
use crate::features::al::alpg000::models::AuditStatusDto;

impl Alpg000CommandExecutor {
    pub(super) async fn audit_store_status(&self) -> AuditStatusDto {
        AuditStatusDto { available: self.audit.is_available().await }
    }
}
