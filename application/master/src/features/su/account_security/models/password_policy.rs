//! The password policy: what a new password must look like (`security.password.*` in
//! `su_configuration`). Ory Kratos only knows a minimum length, so the rest is checked by the
//! auth gateway before a password is handed to Kratos.

use std::collections::HashMap;

pub const KEY_MIN_LENGTH: &str = "security.password.min_length";
pub const KEY_REQUIRE_UPPERCASE: &str = "security.password.require_uppercase";
pub const KEY_REQUIRE_LOWERCASE: &str = "security.password.require_lowercase";
pub const KEY_REQUIRE_DIGIT: &str = "security.password.require_digit";
pub const KEY_REQUIRE_SYMBOL: &str = "security.password.require_symbol";
pub const KEY_FORBID_IDENTIFIER: &str = "security.password.forbid_identifier";

/// Kratos itself refuses passwords shorter than this, whatever the policy says
pub const MIN_LENGTH_FLOOR: usize = 8;
pub const MIN_LENGTH_CEILING: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_symbol: bool,
    /// The password may not contain the user name or the e-mail's name part (3 characters or more)
    pub forbid_identifier: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: MIN_LENGTH_FLOOR,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_symbol: false,
            forbid_identifier: true,
        }
    }
}

/// What a password lacks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyViolation {
    TooShort(usize),
    NeedUppercase,
    NeedLowercase,
    NeedDigit,
    NeedSymbol,
    ContainsIdentifier,
}

impl PolicyViolation {
    /// Thai / English text of the rule
    pub fn message(&self) -> String {
        match self {
            Self::TooShort(n) => format!("ยาวอย่างน้อย {n} ตัวอักษร / at least {n} characters"),
            Self::NeedUppercase => "มีตัวพิมพ์ใหญ่ (A-Z) / an uppercase letter".to_string(),
            Self::NeedLowercase => "มีตัวพิมพ์เล็ก (a-z) / a lowercase letter".to_string(),
            Self::NeedDigit => "มีตัวเลข (0-9) / a digit".to_string(),
            Self::NeedSymbol => "มีอักขระพิเศษ เช่น ! @ # / a symbol such as ! @ #".to_string(),
            Self::ContainsIdentifier => "ไม่มีชื่อผู้ใช้หรืออีเมลของคุณอยู่ในรหัสผ่าน / must not contain your user name or e-mail name".to_string(),
        }
    }
}

/// The text shown on the page for a rejected password
pub fn violations_text(violations: &[PolicyViolation]) -> String {
    let rules: Vec<String> = violations.iter().map(PolicyViolation::message).collect();
    format!("รหัสผ่านไม่เป็นไปตามนโยบาย ต้อง: {} / Password does not meet the policy. It needs: {}", rules.join(" • "), rules.join(" • "))
}

impl PasswordPolicy {
    /// Policy from `key -> value` rows; a missing or unreadable value keeps its default, and the
    /// length is kept within what Kratos accepts
    pub fn from_rows(rows: &HashMap<String, String>) -> Self {
        let d = Self::default();
        let flag = |key: &str, default: bool| match rows.get(key).map(|v| v.trim().to_lowercase()) {
            Some(v) if v == "1" || v == "true" => true,
            Some(v) if v == "0" || v == "false" => false,
            _ => default,
        };
        Self {
            min_length: rows
                .get(KEY_MIN_LENGTH)
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(d.min_length)
                .clamp(MIN_LENGTH_FLOOR, MIN_LENGTH_CEILING),
            require_uppercase: flag(KEY_REQUIRE_UPPERCASE, d.require_uppercase),
            require_lowercase: flag(KEY_REQUIRE_LOWERCASE, d.require_lowercase),
            require_digit: flag(KEY_REQUIRE_DIGIT, d.require_digit),
            require_symbol: flag(KEY_REQUIRE_SYMBOL, d.require_symbol),
            forbid_identifier: flag(KEY_FORBID_IDENTIFIER, d.forbid_identifier),
        }
    }

    /// The rules `password` breaks (empty = it is fine). `username` and `email` are the account's.
    pub fn violations(&self, password: &str, username: &str, email: &str) -> Vec<PolicyViolation> {
        let mut found = Vec::new();
        if password.chars().count() < self.min_length {
            found.push(PolicyViolation::TooShort(self.min_length));
        }
        if self.require_uppercase && !password.chars().any(char::is_uppercase) {
            found.push(PolicyViolation::NeedUppercase);
        }
        if self.require_lowercase && !password.chars().any(char::is_lowercase) {
            found.push(PolicyViolation::NeedLowercase);
        }
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            found.push(PolicyViolation::NeedDigit);
        }
        if self.require_symbol && !password.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace()) {
            found.push(PolicyViolation::NeedSymbol);
        }
        if self.forbid_identifier {
            let lower = password.to_lowercase();
            let name_part = email.split('@').next().unwrap_or_default();
            let hit = [username, name_part]
                .iter()
                .map(|n| n.trim().to_lowercase())
                .any(|n| n.chars().count() >= 3 && lower.contains(&n));
            if hit {
                found.push(PolicyViolation::ContainsIdentifier);
            }
        }
        found
    }
}
