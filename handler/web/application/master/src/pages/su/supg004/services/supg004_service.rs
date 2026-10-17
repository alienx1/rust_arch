//! SUPG004 HTTP Service
//!
//! Calls the endpoints of program `SUPG004` under `/api/v1/su/supg004`.

use web_package::core::i18n::t_or;
use crate::api_client::{ApiClient, query_string};
use crate::pages::su::supg004::models::{
    CreateUserInput, CreateUserResultDto, EmployeeOptionDto, GroupOptionDto, PasswordMailResultDto,
    ScopeOptionsDto, UpdateUserInput, UserAccountDto, UserPageDto,
};

/// API base path of program `SUPG004` (= `db_program.path` under `/api/v1`).
pub const SUPG004_API: &str = "/api/v1/su/supg004";

/// Query for the account list; unset / blank / `ALL` values are left out of the URL.
#[derive(Clone, Debug, Default)]
pub struct UserListQuery {
    pub group_id: Option<String>,
    pub search: Option<String>,
    pub active: Option<bool>,
    pub page: u64,
    pub limit: u64,
}

impl UserListQuery {
    fn to_query_string(&self) -> String {
        query_string(vec![
            ("group_id", self.group_id.clone()),
            ("search", self.search.clone()),
            ("active", self.active.map(|a| a.to_string())),
            ("page", Some(self.page.max(1).to_string())),
            ("limit", (self.limit > 0).then(|| self.limit.to_string())),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct Supg004Service {
    api: ApiClient,
}

impl Default for Supg004Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Supg004Service {
    pub fn new() -> Self {
        Self { api: ApiClient::new(SUPG004_API) }
    }

    pub async fn fetch_users(&self, q: &UserListQuery) -> Result<UserPageDto, String> {
        self.api.get(&format!("/users{}", q.to_query_string()), &t_or("supg004.what.user", "บัญชีผู้ใช้")).await
    }

    pub async fn fetch_user(&self, id: &str) -> Result<UserAccountDto, String> {
        self.api.get(&format!("/users/{id}"), &t_or("supg004.what.user", "บัญชีผู้ใช้")).await
    }

    pub async fn fetch_groups(&self) -> Result<Vec<GroupOptionDto>, String> {
        self.api.get("/options/groups", &t_or("supg004.what.group", "กลุ่มสิทธิ์")).await
    }

    pub async fn fetch_scope_options(&self) -> Result<ScopeOptionsDto, String> {
        self.api.get("/scope-options", &t_or("supg004.what.scope", "ขอบเขตข้อมูล")).await
    }

    pub async fn fetch_employees(&self, search: &str) -> Result<Vec<EmployeeOptionDto>, String> {
        let q = query_string(vec![("search", Some(search.to_string()))]);
        self.api.get(&format!("/options/employees{q}"), &t_or("supg004.what.employee", "พนักงาน")).await
    }

    pub async fn create_user(&self, i: CreateUserInput) -> Result<CreateUserResultDto, String> {
        self.api.post_cbor("/users", &i, &t_or("supg004.what.user", "บัญชีผู้ใช้")).await
    }

    pub async fn update_user(&self, id: &str, i: UpdateUserInput) -> Result<UserAccountDto, String> {
        self.api.put_cbor(&format!("/users/{id}"), &i, &t_or("supg004.what.user", "บัญชีผู้ใช้")).await
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), String> {
        self.api.delete(&format!("/users/{id}"), &t_or("supg004.what.user", "บัญชีผู้ใช้")).await
    }

    /// Mails the user a link to set a new password (the old one keeps working until then)
    pub async fn send_password_mail(&self, id: &str) -> Result<PasswordMailResultDto, String> {
        self.api.post_cbor(&format!("/users/{id}/password-mail"), &(), &t_or("supg004.action.password_mail", "อีเมลตั้งรหัสผ่าน")).await
    }

    /// Signs the user out everywhere and blocks sign-in until a new password is set; mails the link
    pub async fn force_password_change(&self, id: &str) -> Result<PasswordMailResultDto, String> {
        self.api.post_cbor(&format!("/users/{id}/force-password-change"), &(), &t_or("supg004.action.force_change", "บังคับเปลี่ยนรหัสผ่าน")).await
    }
}
