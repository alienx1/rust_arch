//! SUPG004 DTOs: user accounts, selector options and write payloads

use serde::{Deserialize, Serialize};
use web_package::core::i18n::{localized, t_or};

use crate::pages::db::dbpg000::models::{deserialize_id, deserialize_opt_id, serialize_id};

/// Scope levels an assignment can have, as the API names them (hierarchy order)
pub const SCOPE_LEVELS: [&str; 4] = ["organization", "company", "branch", "department"];

pub fn scope_level_label(level: &str) -> String {
    match level {
        "organization" => t_or("supg004.scope.organization", "องค์กร (Organization)"),
        "company" => t_or("supg004.scope.company", "บริษัท (Company)"),
        "branch" => t_or("supg004.scope.branch", "สาขา (Branch)"),
        "department" => t_or("supg004.scope.department", "แผนก (Department)"),
        _ => "-".to_string(),
    }
}

/// Hierarchy order of the scope levels (organization → company → branch → department)
pub fn scope_level_index(level: &str) -> usize {
    SCOPE_LEVELS.iter().position(|k| *k == level).unwrap_or(0)
}

/// A permission group an account holds at one scope unit
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAssignmentDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub group_id: String,
    pub group_code: String,
    pub scope_level: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub scope_id: String,
    pub scope_code: String,
    pub scope_name: String,
    pub scope_path: String,
}

impl UserAssignmentDto {
    /// `GROUP @ บริษัท (Company): ARCH › C01 — ชื่อ`
    pub fn display_name(&self) -> String {
        format!(
            "{} @ {}: {} — {}",
            self.group_code,
            scope_level_label(&self.scope_level),
            self.scope_path,
            self.scope_name
        )
    }

    pub fn command(&self) -> AssignmentInput {
        AssignmentInput {
            permission_group_id: self.group_id.clone(),
            scope_level: self.scope_level.clone(),
            scope_id: self.scope_id.clone(),
        }
    }
}

/// An assignment sent to the API
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssignmentInput {
    #[serde(serialize_with = "serialize_id")]
    pub permission_group_id: String,
    pub scope_level: String,
    #[serde(serialize_with = "serialize_id")]
    pub scope_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub parent_id: Option<String>,
}

impl ScopeOptionDto {
    pub fn display_name(&self) -> String {
        format!("{} ({})", localized(&self.name_th, &self.name_en), self.code)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScopeOptionsDto {
    pub organizations: Vec<ScopeOptionDto>,
    pub companies: Vec<ScopeOptionDto>,
    pub branches: Vec<ScopeOptionDto>,
    pub departments: Vec<ScopeOptionDto>,
}

impl ScopeOptionsDto {
    /// Units of a level under `parent` (organizations have no parent)
    pub fn children(&self, level: &str, parent: &str) -> Vec<ScopeOptionDto> {
        self.for_level(level)
            .into_iter()
            .filter(|o| level == "organization" || o.parent_id.as_deref() == Some(parent))
            .collect()
    }

    /// Options for a scope level
    pub fn for_level(&self, level: &str) -> Vec<ScopeOptionDto> {
        match level {
            "organization" => self.organizations.clone(),
            "company" => self.companies.clone(),
            "branch" => self.branches.clone(),
            "department" => self.departments.clone(),
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserAccountDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub username: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub employee_id: String,
    pub employee_code: String,
    pub full_name: String,
    pub department_name: String,
    pub company_name: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub group_id: String,
    pub group_code: String,
    /// Groups held at scope units (empty for a legacy account without rows)
    #[serde(default)]
    pub assignments: Vec<UserAssignmentDto>,
    pub active: bool,
    /// The account cannot sign in until a new password is set
    #[serde(default)]
    pub must_change_password: bool,
    pub created_time: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserPageDto {
    pub items: Vec<UserAccountDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeeOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub full_name: String,
    pub department_name: String,
    pub company_name: String,
}

impl EmployeeOptionDto {
    pub fn display_name(&self) -> String {
        format!("{} • {} ({})", self.code, self.full_name, self.department_name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub active: bool,
    pub needs_approval: bool,
}

impl GroupOptionDto {
    pub fn display_name(&self) -> String {
        self.code.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasswordMailResultDto {
    pub sent: bool,
    /// Masked address the mail went to (`s***@arch.local`)
    pub sent_to: Option<String>,
    pub mail_error: Option<String>,
    /// Only present when the mail could not be sent: the administrator hands it over
    pub recovery_link: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateUserResultDto {
    pub user: UserAccountDto,
    pub password_mail: PasswordMailResultDto,
}

/// Values of the account form
#[derive(Clone, Debug, Default)]
pub struct UserFormInput {
    pub employee_id: String,
    pub username: String,
    pub email: String,
    pub assignments: Vec<AssignmentInput>,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserInput {
    #[serde(serialize_with = "serialize_id")]
    pub employee_id: String,
    pub username: String,
    pub email: String,
    pub assignments: Vec<AssignmentInput>,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub assignments: Vec<AssignmentInput>,
    pub active: bool,
    pub row_version: i64,
}

/// Suggested username from an email address (local part, lower-cased, invalid characters dropped)
pub fn username_from_email(email: &str) -> String {
    email
        .split('@')
        .next()
        .unwrap_or_default()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect()
}
