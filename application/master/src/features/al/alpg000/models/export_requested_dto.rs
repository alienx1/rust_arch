//! ALPG000 CSV export: the answer to a request

use serde::{Deserialize, Serialize};

/// The export was queued; the finished file is announced by a push notification and listed under
/// "my reports" (RPPG000) as job `job_id`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportRequestedDto {
    pub job_id: String,
}
