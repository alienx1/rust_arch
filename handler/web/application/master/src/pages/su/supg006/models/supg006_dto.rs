//! SUPG006 DTOs: the settings of `su_configuration` this page edits, and the save payload

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigGroup {
    Login,
    Password,
    PasswordPolicy,
    MailTemplate,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigKind {
    Number,
    Flag,
    Text,
    Body,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigItemDto {
    pub key: String,
    pub group: ConfigGroup,
    pub kind: ConfigKind,
    pub value: String,
    pub default_value: String,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub description: Option<String>,
    pub row_version: i64,
}

/// One changed setting of a save
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigChangeInput {
    pub key: String,
    pub value: String,
    pub row_version: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveConfigInput {
    pub changes: Vec<ConfigChangeInput>,
}

/// The four password mails
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailKind {
    pub code: &'static str,
    pub label_key: &'static str,
    pub label_fallback: &'static str,
    pub hint_key: &'static str,
    pub hint_fallback: &'static str,
}

pub const MAIL_KINDS: [MailKind; 4] = [
    MailKind {
        code: "setup",
        label_key: "supg006.kind.setup",
        label_fallback: "ตั้งรหัสผ่านครั้งแรก",
        hint_key: "supg006.kind.setup.hint",
        hint_fallback: "ส่งเมื่อสร้างบัญชีผู้ใช้ใหม่",
    },
    MailKind {
        code: "reset",
        label_key: "supg006.kind.reset",
        label_fallback: "ตั้งรหัสผ่านใหม่",
        hint_key: "supg006.kind.reset.hint",
        hint_fallback: "ส่งเมื่อผู้ดูแลกด \"ส่งอีเมลตั้งรหัสผ่าน\" (รหัสเดิมยังใช้ได้จนกว่าจะตั้งใหม่)",
    },
    MailKind {
        code: "forced",
        label_key: "supg006.kind.forced",
        label_fallback: "บังคับเปลี่ยนรหัสผ่าน",
        hint_key: "supg006.kind.forced.hint",
        hint_fallback: "ส่งเมื่อผู้ดูแลกด \"บังคับเปลี่ยนรหัสผ่าน\" (ผู้ใช้ถูกออกจากระบบและเข้าไม่ได้จนกว่าจะตั้งใหม่)",
    },
    MailKind {
        code: "required",
        label_key: "supg006.kind.required",
        label_fallback: "ส่งลิงก์ซ้ำเมื่อพยายามเข้าสู่ระบบ",
        hint_key: "supg006.kind.required.hint",
        hint_fallback: "ส่งเมื่อผู้ใช้ที่ถูกบังคับเปลี่ยนรหัสผ่านพยายามล็อกอินอีก",
    },
];

/// Placeholders a template may use
pub const PLACEHOLDERS: [&str; 4] = ["username", "link", "hours", "app_name"];

pub const APP_NAME_KEY: &str = "mail.template.app_name";

/// `mail.template.<kind>.<part>`
pub fn template_key(kind: &str, part: &str) -> String {
    format!("mail.template.{kind}.{part}")
}

/// Label and hint (fallback text) of a number setting, by key
pub fn number_field_text(key: &str) -> (&'static str, &'static str) {
    match key {
        "security.login.max_failed_attempts" => (
            "จำนวนครั้งที่ใส่รหัสผ่านผิดได้",
            "ใส่ผิดครบจำนวนนี้ภายในช่วงเวลานับ บัญชีจะถูกล็อก ใส่ 0 เพื่อปิดการล็อก",
        ),
        "security.login.attempt_window_minutes" => (
            "ช่วงเวลานับการใส่ผิด (นาที)",
            "นับเฉพาะครั้งที่ผิดภายในกี่นาทีล่าสุด",
        ),
        "security.login.lock_minutes" => ("ระยะเวลาล็อกบัญชี (นาที)", "บัญชีที่ถูกล็อกจะเข้าใช้ไม่ได้ แม้ใส่รหัสผ่านถูก จนกว่าจะครบเวลา"),
        "security.password.min_length" => (
            "ความยาวรหัสผ่านขั้นต่ำ (ตัวอักษร)",
            "รหัสผ่านใหม่ต้องยาวอย่างน้อยเท่านี้ (ต่ำสุด 8 เพราะ Kratos ไม่รับสั้นกว่านี้ สูงสุด 128)",
        ),
        "security.password.mail_resend_minutes" => (
            "ส่งลิงก์ตั้งรหัสผ่านซ้ำได้ทุก (นาที)",
            "ผู้ใช้ที่ถูกบังคับเปลี่ยนรหัสผ่านและพยายามล็อกอินจะได้รับลิงก์ซ้ำ อย่างมากหนึ่งครั้งต่อช่วงนี้ (0 = ไม่ส่งซ้ำอัตโนมัติ)",
        ),
        _ => ("", ""),
    }
}

/// Fills the placeholders of a template with sample values for the preview. The values are put in
/// as typed: the preview shows a mail of made-up data in a sandboxed frame.
pub fn render_preview(template: &str, app_name: &str) -> String {
    let app = if app_name.trim().is_empty() { "Arch" } else { app_name.trim() };
    [("username", "somchai.j"), ("link", "https://example.invalid/recovery?token=…"), ("hours", "24"), ("app_name", app)]
        .iter()
        .fold(template.to_string(), |text, (name, value)| text.replace(&format!("{{{{{name}}}}}"), value))
}

/// The on / off rules of the password policy, in the order they are shown, with their texts
pub const POLICY_FLAGS: [(&str, &str, &str); 5] = [
    ("security.password.require_uppercase", "ต้องมีตัวพิมพ์ใหญ่ (A-Z)", "supg006.flag.require_uppercase"),
    ("security.password.require_lowercase", "ต้องมีตัวพิมพ์เล็ก (a-z)", "supg006.flag.require_lowercase"),
    ("security.password.require_digit", "ต้องมีตัวเลข (0-9)", "supg006.flag.require_digit"),
    ("security.password.require_symbol", "ต้องมีอักขระพิเศษ เช่น ! @ #", "supg006.flag.require_symbol"),
    ("security.password.forbid_identifier", "ห้ามมีชื่อผู้ใช้หรืออีเมลอยู่ในรหัสผ่าน", "supg006.flag.forbid_identifier"),
];
