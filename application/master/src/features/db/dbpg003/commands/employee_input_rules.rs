//! DBPG003 input normalization shared by employee and person name writes

use crate::features::db::dbpg003::errors::Dbpg003Error;

pub(super) fn required(value: &str, what: &str) -> Result<String, Dbpg003Error> {
    let v = value.trim();
    if v.is_empty() {
        return Err(Dbpg003Error::Validation(format!("{what} is required")));
    }
    Ok(v.to_string())
}

pub(super) fn optional(value: &str) -> Option<String> {
    let v = value.trim();
    (!v.is_empty()).then(|| v.to_string())
}
