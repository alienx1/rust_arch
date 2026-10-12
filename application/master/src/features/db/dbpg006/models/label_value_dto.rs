//! DBPG006 translation of one entry in one language

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabelValueDto {
    pub language_id: Uuid,
    pub language_code: String,
    pub value: String,
}
