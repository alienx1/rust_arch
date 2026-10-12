//! SUPG002 DTOs: program (menu) management

use serde::{Deserialize, Serialize};

use crate::pages::db::dbpg000::models::{
    deserialize_id, deserialize_opt_id, serialize_id, serialize_opt_id,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub system_id: String,
    pub system_code: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub module_id: String,
    pub module_code: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub parent_id: Option<String>,
    pub parent_code: String,
    pub code: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub child_count: u64,
    pub is_protected: bool,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProgramPageDto {
    pub items: Vec<ProgramDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

impl SystemOptionDto {
    pub fn display_name(&self) -> String {
        let name = web_package::core::i18n::localized(&self.name_th, &self.name_en);
        format!("{name} ({})", self.code)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub system_id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

impl ModuleOptionDto {
    pub fn display_name(&self) -> String {
        let name = web_package::core::i18n::localized(&self.name_th, &self.name_en);
        format!("{name} ({})", self.code)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgramOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub system_id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub module_id: String,
    #[serde(default, deserialize_with = "deserialize_opt_id")]
    pub parent_id: Option<String>,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

impl ProgramOptionDto {
    pub fn display_name(&self) -> String {
        let name = web_package::core::i18n::localized(&self.name_th, &self.name_en);
        format!("{} • {name}", self.code)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProgramOptionsDto {
    pub systems: Vec<SystemOptionDto>,
    pub modules: Vec<ModuleOptionDto>,
    pub programs: Vec<ProgramOptionDto>,
}

impl ProgramOptionsDto {
    /// Ids of `id` and everything below it: none of them may become `id`'s parent
    pub fn subtree_of(&self, id: &str) -> Vec<String> {
        let mut found = vec![id.to_string()];
        let mut i = 0;
        while i < found.len() {
            let current = found[i].clone();
            for p in &self.programs {
                if p.parent_id.as_deref() == Some(current.as_str()) && !found.contains(&p.id) {
                    found.push(p.id.clone());
                }
            }
            i += 1;
        }
        found
    }
}

/// Values of the program form
#[derive(Clone, Debug, Default)]
pub struct ProgramFormInput {
    pub system_id: String,
    pub module_id: String,
    pub parent_id: String,
    pub code: String,
    pub path: String,
    pub icon: String,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProgramInput {
    #[serde(serialize_with = "serialize_id")]
    pub system_id: String,
    #[serde(serialize_with = "serialize_id")]
    pub module_id: String,
    #[serde(serialize_with = "serialize_opt_id")]
    pub parent_id: Option<String>,
    pub code: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProgramInput {
    #[serde(serialize_with = "serialize_opt_id")]
    pub parent_id: Option<String>,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
