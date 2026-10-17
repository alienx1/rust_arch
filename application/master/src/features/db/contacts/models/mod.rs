//! Contact models: what kinds exist, who may have them, and the rules a value must follow

mod contact_dto;
mod contact_rules;

pub use contact_dto::{ContactDto, ContactInput};
pub use contact_rules::{ContactKind, ContactOwner, MAX_CONTACTS, normalize_contacts};
