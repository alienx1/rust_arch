//! DBPG006 active language (one input per language in the label form)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageOptionDto {
    pub id: Uuid,
    pub code: String,
    pub name_th: String,
    pub name_en: String,
}
