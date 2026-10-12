//! Session revocation result DTO

use serde::{Deserialize, Serialize};

/// ผลลัพธ์ของการเพิกถอนเซสชัน (Revocation Response)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevokeResultDto {
    pub success: bool,
    pub message: String,
    pub target_id: String,
}
