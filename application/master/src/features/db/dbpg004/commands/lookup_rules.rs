//! DBPG004 lookup rules: category / code normalization and required label validation

use crate::features::db::dbpg004::errors::Dbpg004Error;

/// Category names are lower snake case (`address_type`), like the existing ones.
pub fn normalize_category(raw: &str) -> Result<String, Dbpg004Error> {
    let c = raw.trim().to_lowercase();
    let valid = !c.is_empty()
        && c.len() <= 100
        && c.chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');
    if !valid {
        return Err(Dbpg004Error::Validation(
            "Category must be lower-case letters, digits and underscores (e.g. payment_method)"
                .to_string(),
        ));
    }
    Ok(c)
}

/// Codes are stored upper case (`PUBLIC`, `HEAD`, `001`).
pub fn normalize_code(raw: &str) -> Result<String, Dbpg004Error> {
    let c = raw.trim().to_uppercase();
    if c.is_empty() || c.len() > 100 {
        return Err(Dbpg004Error::Validation(
            "Code is required (max 100 characters)".to_string(),
        ));
    }
    Ok(c)
}

pub(super) fn required_label(value: &str) -> Result<String, Dbpg004Error> {
    let v = value.trim();
    if v.is_empty() {
        return Err(Dbpg004Error::Validation(
            "Thai name is required".to_string(),
        ));
    }
    Ok(v.to_string())
}
