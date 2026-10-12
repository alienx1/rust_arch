//! SUPG004 recovery link DTO

use serde::{Deserialize, Serialize};

/// One-time link to set a password
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryLinkDto {
    pub url: String,
    pub expires_at: Option<String>,
}
