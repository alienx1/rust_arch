//! SUPG002 validation rules shared by program create and update

use crate::features::su::supg002::errors::Supg002Error;

/// Program codes are 3–30 upper-case letters, digits or underscores (`DBPG003`, `ALPG000_GROUP`).
pub fn normalize_program_code(raw: &str) -> Result<String, Supg002Error> {
    let c = raw.trim().to_uppercase();
    let valid = (3..=30).contains(&c.len())
        && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    if !valid {
        return Err(Supg002Error::Validation(
            "Code must be 3-30 letters, digits or underscores (e.g. DBPG006)".to_string(),
        ));
    }
    Ok(c)
}

/// Route of a program page: lower case, starts with `/`, letters/digits/`-`/`_` segments. Blank
/// means "no page" (a folder).
pub fn normalize_program_path(raw: &Option<String>) -> Result<Option<String>, Supg002Error> {
    let Some(raw) = raw.as_deref().map(str::trim).filter(|p| !p.is_empty()) else {
        return Ok(None);
    };
    let p = raw.to_lowercase();
    let segments_ok = p
        .trim_start_matches('/')
        .split('/')
        .all(|seg| seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    let valid = p.starts_with('/')
        && segments_ok
        && !p.contains("//")
        && (p == "/" || !p.ends_with('/'));
    if !valid {
        return Err(Supg002Error::Validation(
            "Path must start with '/' and use letters, digits, '-' and '_' (e.g. /db/dbpg003)".to_string(),
        ));
    }
    Ok(Some(p))
}

pub(super) fn normalize_icon(raw: &Option<String>) -> Result<Option<String>, Supg002Error> {
    match raw.as_deref().map(str::trim).filter(|i| !i.is_empty()) {
        None => Ok(None),
        Some(i) if i.len() <= 100 && i.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') => {
            Ok(Some(i.to_string()))
        }
        Some(_) => Err(Supg002Error::Validation(
            "Icon must be an icon name (letters and digits, max 100 characters)".to_string(),
        )),
    }
}

pub(super) fn thai_name(raw: &str) -> Result<String, Supg002Error> {
    let v = raw.trim();
    if v.is_empty() {
        return Err(Supg002Error::Validation("Thai name is required".to_string()));
    }
    Ok(v.to_string())
}

pub(super) fn english_or(raw: &str, thai: &str) -> String {
    match raw.trim() {
        "" => thai.to_string(),
        en => en.to_string(),
    }
}
