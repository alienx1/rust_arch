//! DBPG006 update-label command DTO

use serde::{Deserialize, Serialize};

use super::label_value_input::LabelValueInput;

/// Command DTO to replace the translations of an existing entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateLabelCommand {
    pub category: String,
    pub key: String,
    pub values: Vec<LabelValueInput>,
}
