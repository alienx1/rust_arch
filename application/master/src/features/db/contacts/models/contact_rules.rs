//! What a contact may be: its kinds, who may have which kind, and how a value must look.

use std::collections::HashSet;

use super::{ContactDto, ContactInput};
use crate::features::db::contacts::errors::ContactError;

/// Contacts a record may hold
pub const MAX_CONTACTS: usize = 30;
const MAX_VALUE_CHARS: usize = 200;

/// Kinds of contact (`db_master_lookup.code` of category `contact_type`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactKind {
    Phone,
    Fax,
    Email,
    Line,
    /// Organization / company / branch only
    TaxId,
    /// Employee only
    IdCard,
}

impl ContactKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Phone => "PHONE",
            Self::Fax => "FAX",
            Self::Email => "EMAIL",
            Self::Line => "LINE",
            Self::TaxId => "TAX_ID",
            Self::IdCard => "ID_CARD",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code.trim().to_ascii_uppercase().as_str() {
            "PHONE" => Self::Phone,
            "FAX" => Self::Fax,
            "EMAIL" => Self::Email,
            "LINE" => Self::Line,
            "TAX_ID" => Self::TaxId,
            "ID_CARD" => Self::IdCard,
            _ => return None,
        })
    }

    /// Whether `owner` may have this kind: a tax id belongs to a legal entity or branch, an ID card
    /// number to a person
    pub fn allowed_for(self, owner: ContactOwner) -> bool {
        match self {
            Self::TaxId => owner != ContactOwner::Employee,
            Self::IdCard => owner == ContactOwner::Employee,
            _ => true,
        }
    }

    /// Why `value` is not acceptable for this kind (`None` = fine)
    fn problem(self, value: &str) -> Option<&'static str> {
        let digits = value.chars().filter(char::is_ascii_digit).count();
        match self {
            Self::Phone | Self::Fax => {
                let ok_chars = value.chars().all(|c| c.is_ascii_digit() || "+-() .#xX".contains(c));
                (!(ok_chars && (5..=20).contains(&digits))).then_some("must be a phone number (5-20 digits, + - ( ) . allowed)")
            }
            Self::Email => {
                let valid = value
                    .split_once('@')
                    .is_some_and(|(local, domain)| {
                        !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
                    })
                    && !value.contains(char::is_whitespace)
                    && value.matches('@').count() == 1;
                (!valid).then_some("must be an e-mail address")
            }
            Self::TaxId | Self::IdCard => {
                let ok = (5..=30).contains(&value.chars().count())
                    && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
                (!ok).then_some("must be 5-30 letters, digits or '-'")
            }
            Self::Line => None,
        }
    }
}

/// Who owns a contact group
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactOwner {
    Organization,
    Company,
    Branch,
    Employee,
}

/// Checks and tidies a list of contacts to store: known kinds the owner may have, values that
/// follow their kind, no duplicates, and exactly one default per kind (the first of the kind when
/// none is marked; marking two is an error).
pub fn normalize_contacts(owner: ContactOwner, input: Vec<ContactInput>) -> Result<Vec<ContactDto>, ContactError> {
    if input.len() > MAX_CONTACTS {
        return Err(ContactError::Validation(format!("At most {MAX_CONTACTS} contacts")));
    }
    let mut out: Vec<(ContactKind, ContactDto)> = Vec::with_capacity(input.len());
    let mut seen = HashSet::new();
    for c in input {
        let kind = ContactKind::from_code(&c.type_code)
            .ok_or_else(|| ContactError::Validation(format!("Unknown contact type `{}`", c.type_code)))?;
        if !kind.allowed_for(owner) {
            return Err(ContactError::Validation(format!("{} is not a contact of this record", kind.code())));
        }
        let value = c.value.trim().to_string();
        if value.is_empty() {
            return Err(ContactError::Validation(format!("{} value is required", kind.code())));
        }
        if value.chars().count() > MAX_VALUE_CHARS {
            return Err(ContactError::Validation(format!("{} value is longer than {MAX_VALUE_CHARS} characters", kind.code())));
        }
        if let Some(problem) = kind.problem(&value) {
            return Err(ContactError::Validation(format!("{} {problem}", kind.code())));
        }
        if !seen.insert((kind.code(), value.to_lowercase())) {
            return Err(ContactError::Validation(format!("{} `{value}` is listed twice", kind.code())));
        }
        out.push((kind, ContactDto { type_code: kind.code().to_string(), value, default_contact: c.default_contact }));
    }

    let kinds: HashSet<&'static str> = out.iter().map(|(k, _)| k.code()).collect();
    for kind in kinds {
        let marked = out.iter().filter(|(k, c)| k.code() == kind && c.default_contact).count();
        if marked > 1 {
            return Err(ContactError::Validation(format!("More than one default {kind}")));
        }
        if marked == 0
            && let Some((_, first)) = out.iter_mut().find(|(k, _)| k.code() == kind)
        {
            first.default_contact = true;
        }
    }
    Ok(out.into_iter().map(|(_, c)| c).collect())
}
