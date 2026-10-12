//! SUPG004 identity rules: input validation and identity-provider error mapping shared by the
//! account actions.

use application_core::traits::IdentityError;

use crate::features::su::supg004::errors::Supg004Error;

/// Usernames follow the identity schema: 3–64 letters, digits, `.`, `_` or `-`; stored lower case.
pub fn normalize_username(raw: &str) -> Result<String, Supg004Error> {
    let u = raw.trim().to_lowercase();
    let valid = (3..=64).contains(&u.len())
        && u.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if !valid {
        return Err(Supg004Error::Validation(
            "Username must be 3-64 letters, digits, '.', '_' or '-' (e.g. somchai.j)".to_string(),
        ));
    }
    Ok(u)
}

/// A plain address check (`local@domain.tld`, no spaces); the identity provider validates further.
pub fn is_valid_email(email: &str) -> bool {
    let e = email.trim();
    let Some((local, domain)) = e.split_once('@') else { return false };
    e.len() <= 254
        && !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && !e.chars().any(char::is_whitespace)
}

pub(super) fn provider_error(e: IdentityError) -> Supg004Error {
    match e {
        IdentityError::Conflict(m) => Supg004Error::Duplicate(format!(
            "An identity with this email or username already exists at the identity provider ({m})"
        )),
        IdentityError::NotFound(m) => Supg004Error::NotFound(format!("No identity found for '{m}'")),
        IdentityError::Provider(m) => Supg004Error::IdentityProvider(m),
    }
}
