//! Input rules shared by the geography create / update actions

use crate::features::db::dbpg005::errors::Dbpg005Error;

/// Longest unit code the tables allow (`varchar(10)`)
const MAX_CODE_LEN: usize = 10;
/// Longest postal code the table allows (`varchar(20)`)
const MAX_POSTAL_LEN: usize = 20;

pub(super) fn unit_code(raw: &str) -> Result<String, Dbpg005Error> {
    let c = raw.trim();
    if c.is_empty() || c.len() > MAX_CODE_LEN {
        return Err(Dbpg005Error::Validation(format!(
            "Code is required (max {MAX_CODE_LEN} characters)"
        )));
    }
    Ok(c.to_string())
}

/// Postal code: letters/digits with optional space or dash, e.g. `10110`.
pub fn normalize_postal_code(raw: &str) -> Result<String, Dbpg005Error> {
    let c = raw.trim().to_uppercase();
    let valid = !c.is_empty()
        && c.len() <= MAX_POSTAL_LEN
        && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-');
    if !valid {
        return Err(Dbpg005Error::Validation(format!(
            "Postal code must be letters, digits, space or dash (max {MAX_POSTAL_LEN} characters)"
        )));
    }
    Ok(c)
}

pub(super) fn thai_name(raw: &str) -> Result<String, Dbpg005Error> {
    let v = raw.trim();
    if v.is_empty() {
        return Err(Dbpg005Error::Validation("Thai name is required".to_string()));
    }
    Ok(v.to_string())
}

/// English name on create: falls back to the Thai name when blank.
pub(super) fn english_or(raw: &str, thai: &str) -> String {
    match raw.trim() {
        "" => thai.to_string(),
        en => en.to_string(),
    }
}
