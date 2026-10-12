//! Time display of audit events: RFC 3339 in, local date/time and UTC text out.

use chrono::{DateTime, Utc};
use wasm_bindgen::JsValue;

/// Local (browser time zone) date and time of an RFC 3339 instant, plus its UTC text.
#[derive(Clone, Debug, PartialEq)]
pub struct EventTime {
    /// `2026-10-09`
    pub date: String,
    /// `14:03:07`
    pub time: String,
    /// `2026-10-09 07:03:07 UTC`
    pub utc: String,
}

impl EventTime {
    /// `None` when the text is not RFC 3339
    pub fn parse(rfc3339: &str) -> Option<Self> {
        let at: DateTime<Utc> = DateTime::parse_from_rfc3339(rfc3339.trim()).ok()?.with_timezone(&Utc);
        let local = js_sys::Date::new(&JsValue::from_f64(at.timestamp_millis() as f64));
        Some(Self {
            date: format!(
                "{:04}-{:02}-{:02}",
                local.get_full_year(),
                local.get_month() + 1,
                local.get_date()
            ),
            time: format!(
                "{:02}:{:02}:{:02}",
                local.get_hours(),
                local.get_minutes(),
                local.get_seconds()
            ),
            utc: at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
    }

    /// `2026-10-09 14:03:07`
    pub fn local_text(&self) -> String {
        format!("{} {}", self.date, self.time)
    }
}
