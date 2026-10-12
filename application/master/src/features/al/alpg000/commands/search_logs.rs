//! ALPG000 search action

use application_core::traits::AuditError;

use super::alpg000_executor::Alpg000CommandExecutor;
use super::search_rules::normalize_search;
use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{AuditLogFilterQuery, AuditLogPageDto};

impl Alpg000CommandExecutor {
    pub(super) async fn search_logs_action(
        &self,
        filter: AuditLogFilterQuery,
    ) -> Result<AuditLogPageDto, Alpg000Error> {
        let search = normalize_search(filter)?;
        let page = self.audit.search(search).await.map_err(map_store_error)?;
        Ok(AuditLogPageDto {
            items: page.items.into_iter().map(Into::into).collect(),
            total: page.total,
            page: page.page,
            limit: page.limit,
        })
    }
}

/// Store failures as feature errors
pub(super) fn map_store_error(e: AuditError) -> Alpg000Error {
    match e {
        AuditError::Unavailable(m) => Alpg000Error::Unavailable(m),
        AuditError::InvalidQuery(m) => Alpg000Error::Validation(m),
    }
}
