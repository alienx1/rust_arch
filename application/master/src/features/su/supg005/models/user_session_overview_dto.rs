//! User overview with online sessions DTO

use serde::{Deserialize, Serialize};

use super::active_device_dto::ActiveDeviceDto;

/// ข้อมูลภาพรวมของผู้ใช้งานพร้อมเซสชันที่ออนไลน์
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSessionOverviewDto {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub employee_code: String,
    pub role_code: String,
    pub role_name: String,
    pub scope_level: String,
    pub department_name: String,
    pub company_name: String,
    pub is_active_account: bool,
    pub devices: Vec<ActiveDeviceDto>,
}

impl UserSessionOverviewDto {
    pub fn is_online(&self) -> bool {
        !self.devices.is_empty()
    }

    pub fn active_sessions_count(&self) -> usize {
        self.devices.len()
    }
}
