//! The e-mail that carries a password link, and the answer of the actions that send it

use std::collections::HashMap;

use application_core::traits::MailMessage;
use serde::{Deserialize, Serialize};

/// Why a password mail is sent
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordMailKind {
    /// A new account: set the first password
    Setup,
    /// An administrator sent a link to choose a new password (the old one still works until then)
    Reset,
    /// An administrator forced a change: the account cannot sign in until it is done
    Forced,
    /// The holder tried to sign in while a change is required: the link again
    Required,
}

impl PasswordMailKind {
    /// Part of the `su_configuration` key of its template: `mail.template.<key>.subject|html|text`
    pub fn key(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Reset => "reset",
            Self::Forced => "forced",
            Self::Required => "required",
        }
    }
}

/// What happened when a password mail was asked for
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordMailResult {
    pub sent: bool,
    /// Where it went, with most of the name hidden (`s***@arch.local`)
    pub sent_to: Option<String>,
    /// Why it could not be sent (no mail server, refused address ...)
    pub mail_error: Option<String>,
    /// The link, handed to the administrator only when the mail could not be sent
    pub recovery_link: Option<String>,
}

/// `s***@arch.local`
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => format!("{}***@{domain}", local.chars().next().unwrap_or('*')),
        None => "***".to_string(),
    }
}

/// The message for `kind`. Subject, HTML and text each come from `templates`
/// (`mail.template.<kind>.subject|html|text` in `su_configuration`) when set, otherwise from the
/// built-in text. Placeholders: `{{username}}`, `{{link}}`, `{{hours}}`, `{{app_name}}`
/// (`mail.template.app_name`); in the HTML the values are escaped.
pub fn password_mail_message(
    kind: PasswordMailKind,
    to: &str,
    username: &str,
    url: &str,
    valid_hours: u32,
    templates: &HashMap<String, String>,
) -> MailMessage {
    let built_in = built_in_password_mail(kind, to, username, url, valid_hours);
    let app_name = templates.get("mail.template.app_name").map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("Arch");
    let hours = valid_hours.to_string();
    let pick = |part: &str, fallback: String, html: bool| -> String {
        match templates.get(&format!("mail.template.{}.{part}", kind.key())).filter(|t| !t.trim().is_empty()) {
            Some(t) => {
                let esc = |v: &str| if html { escape_html(v) } else { v.to_string() };
                render_template(t, &[("username", &esc(username)), ("link", &esc(url)), ("hours", &hours), ("app_name", &esc(app_name))])
            }
            None => fallback,
        }
    };
    MailMessage {
        to: built_in.to,
        subject: pick("subject", built_in.subject, false),
        text: pick("text", built_in.text, false),
        html: Some(pick("html", built_in.html.unwrap_or_default(), true)),
        requested_by: None,
    }
}

/// Replaces every `{{name}}` of `template` with its value
pub fn render_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in values {
        out = out.replace(&format!("{{{{{name}}}}}"), value);
    }
    out
}

/// The built-in message for `kind`, in Thai and English (the account has no language setting before it signs in)
fn built_in_password_mail(kind: PasswordMailKind, to: &str, username: &str, url: &str, valid_hours: u32) -> MailMessage {
    let (subject, why_th, why_en) = match kind {
        PasswordMailKind::Setup => (
            "ตั้งรหัสผ่านบัญชีของคุณ / Set your password",
            "มีการสร้างบัญชีผู้ใช้ให้คุณแล้ว กรุณาตั้งรหัสผ่านก่อนเข้าใช้งาน",
            "An account has been created for you. Please set your password before signing in.",
        ),
        PasswordMailKind::Reset => (
            "ตั้งรหัสผ่านใหม่ / Set a new password",
            "ผู้ดูแลระบบส่งลิงก์ให้คุณตั้งรหัสผ่านใหม่",
            "An administrator sent you a link to choose a new password.",
        ),
        PasswordMailKind::Forced => (
            "ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required",
            "ผู้ดูแลระบบกำหนดให้คุณเปลี่ยนรหัสผ่าน คุณจะเข้าใช้งานไม่ได้จนกว่าจะตั้งรหัสผ่านใหม่",
            "An administrator requires you to change your password. You cannot sign in until you set a new one.",
        ),
        PasswordMailKind::Required => (
            "ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required",
            "คุณต้องตั้งรหัสผ่านใหม่ก่อนจึงจะเข้าใช้งานได้ นี่คือลิงก์ใหม่",
            "You must set a new password before you can sign in. Here is a new link.",
        ),
    };
    let text = format!(
        "{why_th}\n\nชื่อผู้ใช้: {username}\nตั้งรหัสผ่าน (ลิงก์ใช้ได้ {valid_hours} ชั่วโมง และใช้ได้ครั้งเดียว):\n{url}\n\n\
         ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้\n\n---\n\n\
         {why_en}\n\nUsername: {username}\nSet your password (the link is valid for {valid_hours} hours and works once):\n{url}\n\n\
         If you did not expect this e-mail, ignore it.\n"
    );
    let (username_html, url_html) = (escape_html(username), escape_html(url));
    let html = format!(
        "<p>{why_th}</p><p>ชื่อผู้ใช้: <b>{username_html}</b></p>\
         <p><a href=\"{url_html}\">ตั้งรหัสผ่าน</a> (ลิงก์ใช้ได้ {valid_hours} ชั่วโมง และใช้ได้ครั้งเดียว)</p>\
         <p>ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้</p><hr>\
         <p>{why_en}</p><p>Username: <b>{username_html}</b></p>\
         <p><a href=\"{url_html}\">Set your password</a> (valid for {valid_hours} hours, works once)</p>\
         <p>If you did not expect this e-mail, ignore it.</p>"
    );
    MailMessage { to: to.to_string(), subject: subject.to_string(), text, html: Some(html), requested_by: None }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
