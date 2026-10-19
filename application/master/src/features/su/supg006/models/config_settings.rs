//! The settings this program edits: what each key is and what it may hold

use serde::{Deserialize, Serialize};

/// Where a setting is shown
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigGroup {
    /// Sign-in lock (`security.login.*`)
    Login,
    /// Password mails (`security.password.mail_resend_minutes`)
    Password,
    /// What a new password must look like (`security.password.*`)
    PasswordPolicy,
    /// E-mail templates (`mail.template.*`)
    MailTemplate,
}

/// How a value is edited and validated
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigKind {
    Number,
    /// On / off, stored as `1` / `0`
    Flag,
    /// One line of text
    Text,
    /// HTML or plain text body: several lines, must contain `{{link}}` unless empty
    Body,
}

/// One editable key
#[derive(Clone, Copy, Debug)]
pub struct ConfigSetting {
    pub key: &'static str,
    /// `su_configuration.category` of a row created here
    pub category: &'static str,
    pub group: ConfigGroup,
    pub kind: ConfigKind,
    pub default_value: &'static str,
    pub min: Option<i64>,
    pub max: Option<i64>,
    /// Longest text accepted, in characters
    pub max_len: usize,
}

const fn number(key: &'static str, group: ConfigGroup, default_value: &'static str, min: i64, max: i64) -> ConfigSetting {
    ConfigSetting { key, category: "security", group, kind: ConfigKind::Number, default_value, min: Some(min), max: Some(max), max_len: 10 }
}

const fn flag(key: &'static str, group: ConfigGroup, default_value: &'static str) -> ConfigSetting {
    ConfigSetting { key, category: "security", group, kind: ConfigKind::Flag, default_value, min: Some(0), max: Some(1), max_len: 5 }
}

const fn template(key: &'static str, kind: ConfigKind, max_len: usize) -> ConfigSetting {
    ConfigSetting { key, category: "mail_template", group: ConfigGroup::MailTemplate, kind, default_value: "", min: None, max: None, max_len }
}

/// Placeholders a template may use (`{{name}}`)
pub const KNOWN_PLACEHOLDERS: [&str; 4] = ["username", "link", "hours", "app_name"];

pub const CONFIG_SETTINGS: &[ConfigSetting] = &[
    number("security.login.max_failed_attempts", ConfigGroup::Login, "5", 0, 100),
    number("security.login.attempt_window_minutes", ConfigGroup::Login, "15", 1, 1440),
    number("security.login.lock_minutes", ConfigGroup::Login, "15", 1, 10080),
    number("security.password.mail_resend_minutes", ConfigGroup::Password, "5", 0, 1440),
    number("security.password.min_length", ConfigGroup::PasswordPolicy, "8", 8, 128),
    flag("security.password.require_uppercase", ConfigGroup::PasswordPolicy, "1"),
    flag("security.password.require_lowercase", ConfigGroup::PasswordPolicy, "1"),
    flag("security.password.require_digit", ConfigGroup::PasswordPolicy, "1"),
    flag("security.password.require_symbol", ConfigGroup::PasswordPolicy, "0"),
    flag("security.password.forbid_identifier", ConfigGroup::PasswordPolicy, "1"),
    template("mail.template.app_name", ConfigKind::Text, 60),
    template("mail.template.setup.subject", ConfigKind::Text, 200),
    template("mail.template.setup.html", ConfigKind::Body, 100_000),
    template("mail.template.setup.text", ConfigKind::Body, 20_000),
    template("mail.template.reset.subject", ConfigKind::Text, 200),
    template("mail.template.reset.html", ConfigKind::Body, 100_000),
    template("mail.template.reset.text", ConfigKind::Body, 20_000),
    template("mail.template.forced.subject", ConfigKind::Text, 200),
    template("mail.template.forced.html", ConfigKind::Body, 100_000),
    template("mail.template.forced.text", ConfigKind::Body, 20_000),
    template("mail.template.required.subject", ConfigKind::Text, 200),
    template("mail.template.required.html", ConfigKind::Body, 100_000),
    template("mail.template.required.text", ConfigKind::Body, 20_000),
];

pub fn find_setting(key: &str) -> Option<&'static ConfigSetting> {
    CONFIG_SETTINGS.iter().find(|s| s.key == key)
}
