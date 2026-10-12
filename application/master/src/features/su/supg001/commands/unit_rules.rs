//! SUPG001 validation rules shared by the system and module writes

use crate::features::referential_integrity::is_foreign_key_violation;
use crate::features::su::supg001::errors::Supg001Error;

/// System / module codes are 2–10 upper-case letters or digits (`SU`, `SUMT`).
pub fn normalize_unit_code(raw: &str) -> Result<String, Supg001Error> {
    let c = raw.trim().to_uppercase();
    let valid = (2..=10).contains(&c.len()) && c.chars().all(|ch| ch.is_ascii_alphanumeric());
    if !valid {
        return Err(Supg001Error::Validation(
            "Code must be 2-10 letters or digits (e.g. SU, SUMT)".to_string(),
        ));
    }
    Ok(c)
}

pub(super) fn thai_name(raw: &str) -> Result<String, Supg001Error> {
    let v = raw.trim();
    if v.is_empty() {
        return Err(Supg001Error::Validation("Thai name is required".to_string()));
    }
    Ok(v.to_string())
}

pub(super) fn english_or(raw: &str, thai: &str) -> String {
    match raw.trim() {
        "" => thai.to_string(),
        en => en.to_string(),
    }
}

/// A refused delete because other rows still reference the unit is "in use", not a database fault
pub(super) fn in_use_or_database(err: sea_orm::DbErr) -> Supg001Error {
    if is_foreign_key_violation(&err) {
        return Supg001Error::InUse(
            "This record is still referenced by other data and cannot be deleted; switch it off instead"
                .to_string(),
        );
    }
    err.into()
}
