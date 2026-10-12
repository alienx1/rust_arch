//! ALPG000 result page

use serde::{Deserialize, Serialize};

use super::AuditLogSummaryDto;

/// One page of events, newest first
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditLogPageDto {
    pub items: Vec<AuditLogSummaryDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}
