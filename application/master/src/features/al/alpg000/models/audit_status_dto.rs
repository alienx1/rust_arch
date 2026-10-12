//! ALPG000 store status

use serde::{Deserialize, Serialize};

/// `available` is `false` while the audit store (Quickwit) does not answer
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuditStatusDto {
    pub available: bool,
}
