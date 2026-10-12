//! DBPG006 DTOs: i18n label (`db_label`) read models and write payloads

use serde::{Deserialize, Serialize};

use crate::pages::db::dbpg000::models::{deserialize_id, serialize_id};

/// An active language (one input per language in the form)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageOptionDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}

/// A category with its number of entries
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelCategoryDto {
    pub category: String,
    pub entries: u64,
}

/// Translation of one entry in one language
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelValueDto {
    #[serde(deserialize_with = "deserialize_id")]
    pub language_id: String,
    pub language_code: String,
    pub value: String,
}

/// One entry: `(category, key)` with its translations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelEntryDto {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueDto>,
}

/// One page of entries
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LabelPageDto {
    pub items: Vec<LabelEntryDto>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

/// One language's text in a save payload; blank = no translation in that language
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelValueInput {
    #[serde(serialize_with = "serialize_id")]
    pub language_id: String,
    pub value: String,
}

/// Payload to create or update an entry (same shape for both)
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SaveLabelInput {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueInput>,
}
