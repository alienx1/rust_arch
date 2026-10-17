//! The text of a notification in the user's language.
//!
//! The sender stores its own wording (`title` / `body`, whatever language it wrote) and puts the
//! values of the text in `data`. The web app looks up `ntpg000.kind.<kind>.title` / `.body` in the
//! dictionary of the active language and fills the `{name}` placeholders from `data`; a kind
//! without a dictionary entry shows the stored text.

use serde_json::Value;
use web_package::core::i18n::{t_fmt, t_or};

use super::NotificationDto;

impl NotificationDto {
    pub fn title_text(&self) -> String {
        self.text("title", &self.title)
    }

    pub fn body_text(&self) -> String {
        self.text("body", &self.body)
    }

    fn text(&self, part: &str, stored: &str) -> String {
        let args: Vec<(String, String)> = match &self.data {
            Value::Object(map) => map
                .iter()
                .filter_map(|(k, v)| match v {
                    Value::String(s) => Some((k.clone(), s.clone())),
                    Value::Number(n) => Some((k.clone(), n.to_string())),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        let key = format!("ntpg000.kind.{}.{part}", self.kind);
        let template = t_or(&key, "");
        // No entry, or a notification stored before its values were put in `data`: show what the
        // sender wrote rather than a text with holes
        if template.is_empty() || placeholders(&template).any(|name| !args.iter().any(|(k, _)| k == name)) {
            return stored.to_string();
        }
        let args: Vec<(&str, &str)> = args.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        t_fmt(&key, stored, &args)
    }
}

/// The `{name}` placeholders of a template
fn placeholders(template: &str) -> impl Iterator<Item = &str> {
    template.split('{').skip(1).filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
}
