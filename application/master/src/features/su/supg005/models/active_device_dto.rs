//! Active device session DTO

use serde::{Deserialize, Serialize};

/// ข้อมูลอุปกรณ์ที่เชื่อมต่อกับเซสชัน
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveDeviceDto {
    pub id: String,
    pub user_id: String,
    pub device_name: String,
    pub ip_address: String,
    pub location: String,
    pub last_active: String,
    pub authenticated_at: String,
    pub is_current: bool,
}
