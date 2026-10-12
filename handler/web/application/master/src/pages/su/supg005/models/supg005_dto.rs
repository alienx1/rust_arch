//! IAM Session Data Transfer Objects (DTOs) & View Models

use serde::{Deserialize, Serialize};

/// ข้อมูลตัวตนผู้ใช้งาน
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdentityDto {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub employee_code: String,
    pub account_status: String,
}

impl UserIdentityDto {
    pub fn full_name(&self) -> String {
        if self.first_name.is_empty() && self.last_name.is_empty() {
            self.username.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name).trim().to_string()
        }
    }

    pub fn initials(&self) -> String {
        let first = self.first_name.chars().next().unwrap_or('U');
        let last = self.last_name.chars().next().unwrap_or('S');
        format!("{}{}", first, last).to_uppercase()
    }
}

/// ข้อมูลความปลอดภัยของเซสชัน (Ory Kratos Session Security)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSecurityDto {
    pub session_id: String,
    pub is_active: bool,
    pub aal_level: String,
    pub auth_method: String,
    pub authenticated_at: String,
    pub expires_at: String,
    pub remaining_time: String,
    pub ip_address: String,
    pub user_agent: String,
}

/// ตัวเลือกบริษัทสำหรับ Scope Switcher
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyOptionDto {
    pub id: String,
    pub code: String,
    pub name: String,
}

/// ขอบเขตข้อมูลและองค์กร (Active Scope Context)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeContextDto {
    pub scope_level: String,
    pub organization_code: String,
    pub organization_name: String,
    pub company_id: String,
    pub company_code: String,
    pub company_name: String,
    pub branch_code: String,
    pub branch_name: String,
    pub department_code: String,
    pub department_name: String,
    pub permission_group_code: String,
    pub permission_group_name: String,
    pub available_companies: Vec<CompanyOptionDto>,
}

/// สิทธิ์การเข้าถึงเมนูและโปรแกรม (Effective Permission)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramPermissionDto {
    pub program_code: String,
    pub program_name: String,
    pub module_code: String,
    pub module_name: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_approve: bool,
}

/// ข้อมูลอุปกรณ์ที่เชื่อมต่อเซสชัน (Active Device / Session)
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

/// ข้อมูลภาพรวมเซสชันของผู้ใช้งานแต่ละคน (User Session Overview)
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
    pub fn active_sessions_count(&self) -> usize {
        self.devices.len()
    }

    pub fn is_online(&self) -> bool {
        !self.devices.is_empty()
    }

    pub fn initials(&self) -> String {
        let parts: Vec<&str> = self.full_name.split_whitespace().collect();
        let first = parts.first().and_then(|p| p.chars().next()).unwrap_or('U');
        let last = parts.get(1).and_then(|p| p.chars().next()).unwrap_or('S');
        format!("{}{}", first, last).to_uppercase()
    }
}

/// สรุปข้อมูลทั้งหมดของหน้าจอ IAM Session
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Supg005SessionSummary {
    pub identity: UserIdentityDto,
    pub security: SessionSecurityDto,
    pub scope: ScopeContextDto,
    pub permissions: Vec<ProgramPermissionDto>,
    pub devices: Vec<ActiveDeviceDto>,
    pub all_users: Vec<UserSessionOverviewDto>,
    pub raw_json: String,
    pub is_live_session: bool,
}

impl Supg005SessionSummary {
    pub fn total_users_count(&self) -> usize {
        self.all_users.len()
    }

    pub fn total_online_users_count(&self) -> usize {
        self.all_users.iter().filter(|u| u.is_online()).count()
    }

    pub fn total_active_devices_count(&self) -> usize {
        self.all_users.iter().map(|u| u.active_sessions_count()).sum()
    }
}

/// พารามิเตอร์ Pagination และการค้นหาผู้ใช้สำหรับ SQL LIMIT / OFFSET
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub struct UserPaginationQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    pub status: Option<String>,
}

/// โครงสร้างผลลัพธ์แบบแบ่งหน้า (Paginated Result)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}
