//! DBPG002 name/code validation shared by branch and department writes

use crate::features::db::dbpg002::errors::Dbpg002Error;

pub(super) fn required(value: &str, what: &str) -> Result<String, Dbpg002Error> {
    let v = value.trim();
    if v.is_empty() {
        return Err(Dbpg002Error::Validation(format!("{what} is required")));
    }
    Ok(v.to_string())
}

/// Names for a create: English falls back to the Thai name.
pub(super) fn names_for_create(
    name_th: &str,
    name_en: &str,
) -> Result<(String, String), Dbpg002Error> {
    let th = required(name_th, "Thai name")?;
    let en = name_en.trim();
    let en = if en.is_empty() {
        th.clone()
    } else {
        en.to_string()
    };
    Ok((th, en))
}
