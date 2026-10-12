//! SUPG001 DTOs: systems & modules (read models and write payloads)

use web_package::core::i18n::t_or;
use serde::{Deserialize, Serialize};

use crate::pages::db::dbpg000::models::{deserialize_id, serialize_id};

/// What the page manages
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    System,
    Module,
}

impl UnitKind {
    pub const ALL: [UnitKind; 2] = [UnitKind::System, UnitKind::Module];

    /// Segment used in form URLs
    pub fn slug(self) -> &'static str {
        match self {
            UnitKind::System => "system",
            UnitKind::Module => "module",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.slug() == slug)
    }

    pub fn label(self) -> String {
        match self {
            UnitKind::System => t_or("supg001.kind.system", "ระบบ (System)"),
            UnitKind::Module => t_or("supg001.kind.module", "โมดูล (Module)"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub module_count: u64,
    pub program_count: u64,
    pub is_protected: bool,
    pub row_version: i64,
}

impl SystemDto {
    pub fn display_name(&self) -> String {
        let name = web_package::core::i18n::localized(&self.name_th, &self.name_en);
        format!("{name} ({})", self.code)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    #[serde(deserialize_with = "deserialize_id")]
    pub system_id: String,
    pub system_code: String,
    pub system_name: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub program_count: u64,
    pub is_protected: bool,
    pub row_version: i64,
}

/// Values of the form; the save logic picks what the kind needs
#[derive(Clone, Debug, Default)]
pub struct UnitFormInput {
    pub system_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSystemInput {
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateModuleInput {
    #[serde(serialize_with = "serialize_id")]
    pub system_id: String,
    pub code: String,
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateUnitInput {
    pub active: bool,
    pub name_th: String,
    pub name_en: String,
    pub row_version: i64,
}
