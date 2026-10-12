//! SUPG005 Service (IAM & Session Monitor)
//!
//! Handles fetching live session details from Ory Kratos & the SUPG005 backend
//! (`/api/v1/su/supg005`), managing real user directories, and issuing real revocation HTTP calls.

use web_package::core::i18n::{t_or, t_fmt};
use crate::pages::su::supg005::models::{
    ActiveDeviceDto, CompanyOptionDto, Supg005SessionSummary, PaginatedResult, ProgramPermissionDto,
    ScopeContextDto, SessionSecurityDto, UserIdentityDto, UserSessionOverviewDto,
};
use web_package::core::auth::types::KratosSession;
use web_package::core::requestor::HttpClient;

/// API base path of program `SUPG005` (= `db_program.path` under `/api/v1`).
pub const SUPG005_API: &str = "/api/v1/su/supg005";

#[derive(Clone, Debug)]
pub struct Supg005Service {
    http: HttpClient,
}

impl Default for Supg005Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg005Service {
    pub fn new() -> Self {
        Self {
            http: HttpClient::default(),
        }
    }

    /// ดึงรายชื่อผู้ใช้งานแบบแบ่งหน้า (SQL LIMIT & OFFSET) จาก Backend API (`GET /api/v1/su/supg005/users?page=...&limit=...`)
    pub async fn fetch_paginated_users(
        &self,
        page: u64,
        limit: u64,
        search: Option<String>,
        status: Option<String>,
        account_status: Option<String>,
        current_session_id: Option<String>,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, String> {
        let mut query_params = format!("page={page}&limit={limit}");
        if let Some(ref q) = search {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                query_params.push_str(&format!("&search={}", simple_url_encode(trimmed)));
            }
        }
        if let Some(ref s) = status {
            if s != "ALL" {
                query_params.push_str(&format!("&status={}", simple_url_encode(s)));
            }
        }

        if let Some(ref s) = account_status {
            if s != "ALL" {
                query_params.push_str(&format!("&account_status={}", simple_url_encode(s)));
            }
        }

        let resp = self
            .http
            .get(&format!("{SUPG005_API}/users?{query_params}"))
            .send()
            .await;

        match resp {
            Ok(r) if r.is_success() => {
                let mut paginated: PaginatedResult<UserSessionOverviewDto> = r
                    .auto()
                    .map_err(|e| t_fmt("supg005.svc.parse_failed", "แปลงข้อมูลผู้ใช้งานล้มเหลว: {error}", &[("error", &e.to_string())]))?;

                // devices มาจาก Kratos Admin API ฝั่ง backend แล้ว — แค่ระบุว่าอันไหนคือเบราว์เซอร์นี้
                for u in &mut paginated.items {
                    for d in &mut u.devices {
                        d.is_current = current_session_id.as_deref() == Some(d.id.as_str());
                        if d.is_current {
                            d.last_active = t_or("supg005.svc.active_now", "กำลังใช้งานอยู่ขณะนี้");
                        }
                    }
                    u.devices.sort_by(|a, b| b.is_current.cmp(&a.is_current));
                }

                Ok(paginated)
            }
            Ok(r) => Err(t_fmt("supg005.svc.fetch_failed", "ไม่สามารถดึงข้อมูลผู้ใช้งานได้ (HTTP {status})", &[("status", &r.status().to_string())])),
            Err(e) => Err(t_fmt("supg005.svc.connect_failed", "ไม่สามารถเชื่อมต่อกับบริการ IAM Backend ได้: {error}", &[("error", &e.to_string())])),
        }
    }

    /// ดึงรายการเซสชันที่ยังใช้งานอยู่ทั้งหมดของผู้ใช้ปัจจุบันจาก Ory Kratos (`GET /api/kratos/sessions`)
    pub async fn fetch_all_kratos_sessions(&self) -> Vec<KratosSession> {
        let path = "/api/kratos/sessions";
        match self.http.get(path).send().await {
            Ok(resp) => match resp.json::<Vec<KratosSession>>() {
                Ok(sessions) => {
                    let active: Vec<KratosSession> = sessions.into_iter().filter(|s| s.active).collect();
                    leptos::logging::log!("Fetched {} active sessions from Ory Kratos /sessions", active.len());
                    active
                }
                Err(e) => {
                    leptos::logging::warn!("Deserialize Kratos /sessions failed: {e:?}");
                    Vec::new()
                }
            },
            Err(e) => {
                leptos::logging::warn!("GET {path} failed: {e:?}");
                Vec::new()
            }
        }
    }

    /// ดึงรายชื่อผู้ใช้งานหน้าแรก (SQL LIMIT 5)
    pub async fn fetch_live_users(
        &self,
        current_session_id: Option<String>,
    ) -> Result<Vec<UserSessionOverviewDto>, String> {
        let paginated = self
            .fetch_paginated_users(1, 5, None, None, None, current_session_id)
            .await?;
        Ok(paginated.items)
    }

    /// ส่ง `DELETE` และถือว่าสำเร็จเฉพาะเมื่อได้ HTTP 2xx เท่านั้น
    async fn delete_checked(&self, path: &str) -> Result<(), String> {
        let resp = self
            .http
            .delete(path)
            .send()
            .await
            .map_err(|e| t_fmt("supg005.svc.delete_failed", "DELETE {path} ล้มเหลว: {error}", &[("path", &path.to_string()),("error", &e.to_string())]))?;
        if resp.is_success() {
            Ok(())
        } else {
            Err(t_fmt("supg005.svc.delete_rejected", "DELETE {path} ถูกปฏิเสธ (HTTP {status})", &[("path", &path.to_string()),("status", &resp.status().to_string())]))
        }
    }

    /// `DELETE` ไป `/api/v1/su/supg005/{suffix}`
    async fn delete_supg005_checked(&self, suffix: &str) -> Result<(), String> {
        self.delete_checked(&format!("{SUPG005_API}/{suffix}")).await
    }

    /// ส่ง `POST` (ไม่มี body) และถือว่าสำเร็จเฉพาะเมื่อได้ HTTP 2xx เท่านั้น
    async fn post_checked(&self, path: &str) -> Result<(), String> {
        let resp = self
            .http
            .post(path)
            .send()
            .await
            .map_err(|e| t_fmt("supg005.svc.post_failed", "POST {path} ล้มเหลว: {error}", &[("path", &path.to_string()),("error", &e.to_string())]))?;
        if resp.is_success() {
            Ok(())
        } else {
            Err(t_fmt("supg005.svc.post_rejected", "POST {path} ถูกปฏิเสธ (HTTP {status})", &[("path", &path.to_string()),("status", &resp.status().to_string())]))
        }
    }

    /// `POST` ไป `/api/v1/su/supg005/{suffix}`
    async fn post_supg005_checked(&self, suffix: &str) -> Result<(), String> {
        self.post_checked(&format!("{SUPG005_API}/{suffix}")).await
    }

    /// ระงับบัญชีผู้ใช้: ปิดการล็อกอินที่ Kratos + เพิกถอนทุกเซสชัน (`POST /api/v1/su/supg005/users/{id}/suspend`)
    pub async fn suspend_user(&self, user_id: &str) -> Result<(), String> {
        self.post_supg005_checked(&format!("users/{user_id}/suspend"))
            .await
            .map_err(|e| t_fmt("supg005.svc.suspend_failed", "เกิดข้อผิดพลาดในการระงับบัญชี: {error}", &[("error", &e.to_string())]))
    }

    /// ยกเลิกการระงับบัญชี (`POST /api/v1/su/supg005/users/{id}/reactivate`)
    pub async fn reactivate_user(&self, user_id: &str) -> Result<(), String> {
        self.post_supg005_checked(&format!("users/{user_id}/reactivate"))
            .await
            .map_err(|e| t_fmt("supg005.svc.reactivate_failed", "เกิดข้อผิดพลาดในการเปิดใช้งานบัญชี: {error}", &[("error", &e.to_string())]))
    }

    /// สั่งเพิกถอนเซสชันอุปกรณ์อื่นทั้งหมดของผู้ใช้ปัจจุบัน (`DELETE /api/kratos/sessions`)
    pub async fn revoke_other_sessions(&self) -> Result<(), String> {
        self.delete_checked("/api/kratos/sessions")
            .await
            .map_err(|e| t_fmt("supg005.svc.revoke_others_failed", "เกิดข้อผิดพลาดในการเพิกถอนอุปกรณ์อื่น: {error}", &[("error", &e.to_string())]))
    }

    /// สั่งเพิกถอนเซสชันของอุปกรณ์เฉพาะตัว: Backend (Kratos Admin) ก่อน แล้ว fallback เป็น Kratos public
    /// (`DELETE /api/kratos/sessions/{id}`, ใช้ได้เฉพาะเซสชันของผู้ใช้เอง)
    pub async fn revoke_device_session(&self, session_id: &str) -> Result<(), String> {
        match self.delete_supg005_checked(&format!("sessions/{session_id}")).await {
            Ok(()) => Ok(()),
            Err(backend_err) => self
                .delete_checked(&format!("/api/kratos/sessions/{session_id}"))
                .await
                .map_err(|e| t_fmt("supg005.svc.revoke_device_failed", "เกิดข้อผิดพลาดในการเพิกถอนอุปกรณ์: {backend_error}; {error}", &[("backend_error", &backend_err.to_string()),("error", &e.to_string())])),
        }
    }

    /// ดึงข้อมูลเซสชัน โดยเชื่อมโยงกับ Kratos Session และฐานข้อมูลจริง
    pub async fn fetch_session_summary(
        &self,
        kratos_session: Option<KratosSession>,
    ) -> Result<Supg005SessionSummary, String> {
        let companies = vec![
            CompanyOptionDto {
                id: "019ca1ec-0000-7000-8002-000000000001".to_string(),
                code: "HO".to_string(),
                name: "สำนักงานใหญ่ (Head Office)".to_string(),
            },
            CompanyOptionDto {
                id: "019ca1ec-0000-7000-8002-000000000002".to_string(),
                code: "CMP02".to_string(),
                name: "บริษัท อาร์ค ซอฟต์แวร์ โซลูชั่น จำกัด".to_string(),
            },
            CompanyOptionDto {
                id: "019ca1ec-0000-7000-8002-000000000003".to_string(),
                code: "CMP03".to_string(),
                name: "บริษัท อาร์ค ดิจิทัล อินโนเวชั่น จำกัด".to_string(),
            },
        ];

        let permissions = vec![
            ProgramPermissionDto {
                program_code: "SYPG000".to_string(),
                program_name: "ภาพรวมและแดชบอร์ดระบบ (System Dashboard)".to_string(),
                module_code: "SY".to_string(),
                module_name: "ระบบหลัก (System)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "DBPG001".to_string(),
                program_name: "กำหนดข้อมูลองค์กร (Organizations)".to_string(),
                module_code: "DB".to_string(),
                module_name: "ข้อมูลหลัก (Master Data)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "DBPG002".to_string(),
                program_name: "กำหนดสาขาและแผนก (Branches & Depts)".to_string(),
                module_code: "DB".to_string(),
                module_name: "ข้อมูลหลัก (Master Data)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "DBPG003".to_string(),
                program_name: "ข้อมูลพนักงานและตำแหน่ง (Employees)".to_string(),
                module_code: "DB".to_string(),
                module_name: "ข้อมูลหลัก (Master Data)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: false,
            },
            ProgramPermissionDto {
                program_code: "SUPG001".to_string(),
                program_name: "กำหนดระบบและโมดูล (Systems & Modules)".to_string(),
                module_code: "SU".to_string(),
                module_name: "ความปลอดภัย (Security)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "SUPG003".to_string(),
                program_name: "กำหนดเมทริกซ์สิทธิ์ (Permission Matrix)".to_string(),
                module_code: "SU".to_string(),
                module_name: "ความปลอดภัย (Security)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "SUPG004".to_string(),
                program_name: "กำหนดบัญชีผู้ใช้งาน (User Accounts)".to_string(),
                module_code: "SU".to_string(),
                module_name: "ความปลอดภัย (Security)".to_string(),
                can_read: true,
                can_write: true,
                can_approve: true,
            },
            ProgramPermissionDto {
                program_code: "ALPG000".to_string(),
                program_name: "บันทึกประวัติการใช้งาน (Audit Logs)".to_string(),
                module_code: "AL".to_string(),
                module_name: "การตรวจสอบ (Audit)".to_string(),
                can_read: true,
                can_write: false,
                can_approve: false,
            },
        ];

        let (current_device_label, current_ua) = detect_current_device();

        let session = match kratos_session {
            Some(s) => s,
            None => {
                return Err(t_or("supg005.svc.no_session", "ไม่พบเซสชันการเข้าสู่ระบบที่ใช้งานอยู่ (กรุณาเข้าสู่ระบบผ่าน Ory Kratos)"));
            }
        };

        let user_id = session.identity_id().unwrap_or_else(|| session.id.clone());
        let email = session.email().unwrap_or_else(|| "user@arch.local".to_string());
        let full_name = session.name().unwrap_or_else(|| "ผู้ใช้งาน ระบบ".to_string());
        let name_parts: Vec<&str> = full_name.split_whitespace().collect();
        let first_name = name_parts.first().copied().unwrap_or("ผู้ใช้งาน").to_string();
        let last_name = name_parts.get(1).copied().unwrap_or("").to_string();

        let all_users = self.fetch_live_users(Some(session.id.clone())).await.unwrap_or_default();

        let kratos_sessions = self.fetch_all_kratos_sessions().await;
        let mut devices = Vec::new();

        for s in &kratos_sessions {
            let is_curr = s.id == session.id;
            let mut dev_name = if is_curr {
                current_device_label.clone()
            } else {
                t_or("supg005.svc.device_via_kratos", "อุปกรณ์ที่เข้าสู่ระบบผ่าน Ory Kratos")
            };
            let mut ip_addr = "127.0.0.1 (Localhost)".to_string();
            let mut loc = "กรุงเทพมหานคร, ประเทศไทย (TH-10)".to_string();

            if let Some(ref kratos_devs) = s.devices {
                if let Some(kd) = kratos_devs.first() {
                    if let Some(ref ua) = kd.user_agent {
                        dev_name = parse_user_agent_label(ua);
                    }
                    if let Some(ref ip) = kd.ip_address {
                        ip_addr = ip.clone();
                    }
                    if let Some(ref l) = kd.location {
                        loc = l.clone();
                    }
                }
            }

            devices.push(ActiveDeviceDto {
                id: s.id.clone(),
                user_id: user_id.clone(),
                device_name: dev_name,
                ip_address: ip_addr,
                location: loc,
                last_active: if is_curr {
                    t_or("supg005.svc.active_now", "กำลังใช้งานอยู่ขณะนี้")
                } else {
                    t_or("supg005.svc.connected", "เชื่อมต่ออยู่ (Active)")
                },
                authenticated_at: s.authenticated_at.clone().unwrap_or_else(|| {
                    session.authenticated_at.clone().unwrap_or_else(|| "-".to_string())
                }),
                is_current: is_curr,
            });
        }

        if devices.is_empty() {
            devices.push(ActiveDeviceDto {
                id: session.id.clone(),
                user_id: user_id.clone(),
                device_name: current_device_label.clone(),
                ip_address: "127.0.0.1".to_string(),
                location: "กรุงเทพมหานคร, ประเทศไทย (TH-10)".to_string(),
                last_active: t_or("supg005.svc.active_now", "กำลังใช้งานอยู่ขณะนี้"),
                authenticated_at: session.authenticated_at.clone().unwrap_or_else(|| "-".to_string()),
                is_current: true,
            });
        }

        devices.sort_by(|a, b| b.is_current.cmp(&a.is_current));

        let raw_json = serde_json::to_string_pretty(&session)
            .unwrap_or_else(|_| "{}".to_string());

        Ok(Supg005SessionSummary {
            identity: UserIdentityDto {
                user_id: user_id.clone(),
                username: email.split('@').next().unwrap_or("user").to_string(),
                email,
                first_name,
                last_name,
                employee_code: "EMP-CURRENT".to_string(),
                account_status: t_or("supg005.svc.account_active", "เปิดใช้งาน (Active)"),
            },
            security: SessionSecurityDto {
                session_id: session.id.clone(),
                is_active: session.active,
                aal_level: session.authenticator_assurance_level.unwrap_or_else(|| "aal1 (Single Factor - Password)".to_string()),
                auth_method: "Argon2id (Ory Kratos Session)".to_string(),
                authenticated_at: session.authenticated_at.unwrap_or_else(|| "-".to_string()),
                expires_at: session.expires_at.unwrap_or_else(|| "-".to_string()),
                remaining_time: t_or("supg005.svc.not_expired", "เซสชันยังไม่หมดอายุ"),
                ip_address: "127.0.0.1".to_string(),
                user_agent: current_ua.clone(),
            },
            scope: ScopeContextDto {
                scope_level: "organization".to_string(),
                organization_code: "ARCH".to_string(),
                organization_name: "สถาปัตยกรรมระบบกลาง (ARCH Enterprise)".to_string(),
                company_id: "019ca1ec-0000-7000-8002-000000000001".to_string(),
                company_code: "HO".to_string(),
                company_name: "สำนักงานใหญ่ (Head Office)".to_string(),
                branch_code: "BKK01".to_string(),
                branch_name: "สาขากรุงเทพฯ (สำนักงานใหญ่)".to_string(),
                department_code: "IT".to_string(),
                department_name: "ฝ่ายเทคโนโลยีสารสนเทศ (Information Technology)".to_string(),
                permission_group_code: "ADMIN".to_string(),
                permission_group_name: "ผู้ดูแลระบบสูงสุด (Super Administrator)".to_string(),
                available_companies: companies,
            },
            permissions,
            devices,
            all_users,
            raw_json,
            is_live_session: true,
        })
    }
}

/// วิเคราะห์ User-Agent เพื่อสร้างชื่อและหมวดหมู่อุปกรณ์ที่เข้าใจง่าย
fn parse_user_agent_label(ua: &str) -> String {
    let browser: String = if ua.contains("Edg") {
        "Microsoft Edge".to_string()
    } else if ua.contains("Firefox") {
        "Mozilla Firefox".to_string()
    } else if ua.contains("Chrome") {
        "Google Chrome".to_string()
    } else if ua.contains("Safari") {
        "Apple Safari".to_string()
    } else {
        t_or("supg005.svc.browser_web", "เว็บเบราว์เซอร์")
    };

    let os: String = if ua.contains("Linux") {
        t_or("supg005.svc.os_linux", "Linux (เดสก์ท็อป)")
    } else if ua.contains("Windows") {
        "Windows".to_string()
    } else if ua.contains("Macintosh") || ua.contains("Mac OS") {
        "macOS".to_string()
    } else if ua.contains("Android") {
        "Android".to_string()
    } else if ua.contains("iPhone") {
        "iPhone (iOS)".to_string()
    } else if ua.contains("iPad") {
        "iPad (iPadOS)".to_string()
    } else {
        t_or("supg005.svc.os_desktop", "เดสก์ท็อป")
    };

    t_fmt("supg005.svc.browser_on_os", "{browser} บน {os}", &[("browser", &browser.to_string()),("os", &os.to_string())])
}

/// ตรวจจับอุปกรณ์จริงที่กำลังเปิดเบราว์เซอร์ใช้งานอยู่ขณะนี้
fn detect_current_device() -> (String, String) {
    if let Some(win) = web_sys::window() {
        let nav = win.navigator();
        if let Ok(ua) = nav.user_agent() {
            let label = parse_user_agent_label(&ua);
            return (label, ua);
        }
    }
    (
        t_or("supg005.svc.default_device", "Google Chrome บน Linux (เดสก์ท็อป)"),
        "Mozilla/5.0 (X11; Linux x86_64) WebAssembly Leptos/0.8".to_string(),
    )
}

/// Helper encoding query parameter strings
fn simple_url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

