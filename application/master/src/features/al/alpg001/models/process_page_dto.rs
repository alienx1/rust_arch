//! ALPG001 one page of processes

use serde::{Deserialize, Serialize};

use super::ProcessDto;

/// How many processes are in each status under the filter (the status itself left out), for the
/// status chips
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessCountsDto {
    pub queued: u64,
    pub running: u64,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessPageDto {
    pub items: Vec<ProcessDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub counts: ProcessCountsDto,
}
