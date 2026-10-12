//! ALPG001 the answer to a reprocess request

use serde::{Deserialize, Serialize};

/// The process was queued again; its row shows the new outcome when the worker is done
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReprocessRequestedDto {
    pub id: String,
}
