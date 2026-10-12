//! DBPG006 one language's text in a write command

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A blank `value` means "no translation in this language" (the row is removed)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelValueInput {
    pub language_id: Uuid,
    pub value: String,
}
