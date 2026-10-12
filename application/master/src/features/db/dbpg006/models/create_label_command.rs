//! DBPG006 create-label command DTO

use serde::{Deserialize, Serialize};

use super::label_value_input::LabelValueInput;

/// Command DTO to create an entry; `(category, key)` must not exist yet. A new `category` name
/// creates the category.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateLabelCommand {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueInput>,
}
