//! A contact as read and as written

use serde::{Deserialize, Serialize};

/// A contact of a record
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactDto {
    /// `contact_type` code: PHONE, FAX, EMAIL, LINE, TAX_ID or ID_CARD
    pub type_code: String,
    pub value: String,
    /// The preferred contact of its type
    pub default_contact: bool,
}

/// A contact to store (the whole list of a record is replaced at once)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactInput {
    pub type_code: String,
    pub value: String,
    #[serde(default)]
    pub default_contact: bool,
}
