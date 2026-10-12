//! DBPG006 label rules: category / key / value normalization and write-set validation

use std::collections::HashSet;
use uuid::Uuid;

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::{
    LabelValueInput, MAX_CATEGORY_LEN, MAX_KEY_LEN, MAX_VALUE_LEN,
};

/// Categories are trimmed, 1-100 characters of `[A-Za-z0-9_.-]` (case is kept: `menu`, `error`).
pub fn normalize_label_category(raw: &str) -> Result<String, Dbpg006Error> {
    let c = raw.trim();
    let valid = !c.is_empty()
        && c.chars().count() <= MAX_CATEGORY_LEN
        && c.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'));
    if !valid {
        return Err(Dbpg006Error::Validation(format!(
            "Category must be 1-{MAX_CATEGORY_LEN} characters of letters, digits, '_', '.' and '-'"
        )));
    }
    Ok(c.to_string())
}

/// Keys are trimmed and non-empty (at most 300 characters), e.g. `column.thai_name`.
pub fn normalize_label_key(raw: &str) -> Result<String, Dbpg006Error> {
    let k = raw.trim();
    if k.is_empty() || k.chars().count() > MAX_KEY_LEN {
        return Err(Dbpg006Error::Validation(format!(
            "Key is required (max {MAX_KEY_LEN} characters)"
        )));
    }
    Ok(k.to_string())
}

/// Trimmed text per language; `None` for a blank value (= no translation). The set needs at least
/// one text and each language may appear only once.
pub(super) fn normalize_values(
    values: &[LabelValueInput],
) -> Result<Vec<(Uuid, Option<String>)>, Dbpg006Error> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(values.len());
    for v in values {
        if !seen.insert(v.language_id) {
            return Err(Dbpg006Error::Validation(format!(
                "Language {} is given more than once",
                v.language_id
            )));
        }
        let text = v.value.trim();
        if text.chars().count() > MAX_VALUE_LEN {
            return Err(Dbpg006Error::Validation(format!(
                "Text must be at most {MAX_VALUE_LEN} characters"
            )));
        }
        out.push((v.language_id, (!text.is_empty()).then(|| text.to_string())));
    }
    if out.iter().all(|(_, text)| text.is_none()) {
        return Err(Dbpg006Error::Validation(
            "At least one language needs a text".to_string(),
        ));
    }
    Ok(out)
}
