//! Pretty-printed JSON in a scrollable monospace box with a copy button, and a key/value
//! table for header objects.

use leptos::prelude::*;
use serde_json::Value;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Button, ButtonSize, ButtonVariant};

/// Pretty text of a JSON value: strings are parsed as JSON text (raw text when they are not).
pub fn pretty_json(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => match serde_json::from_str::<Value>(text) {
            Ok(parsed) if parsed.is_object() || parsed.is_array() => {
                serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| text.clone())
            }
            _ => text.clone(),
        },
        other => serde_json::to_string_pretty(other).unwrap_or_default(),
    }
}

/// `name: value` pairs of a header object (non-objects give no pairs).
pub fn header_pairs(value: &Option<Value>) -> Vec<(String, String)> {
    let parsed = match value {
        Some(Value::String(text)) => serde_json::from_str::<Value>(text).ok(),
        other => other.clone(),
    };
    match parsed {
        Some(Value::Object(map)) => map
            .into_iter()
            .map(|(k, v)| {
                let text = match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, text)
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Copies `text` to the clipboard (best effort)
fn copy_text(text: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.navigator().clipboard().write_text(text);
    }
}

/// Body viewer; `body` empty/absent shows the "no body" text.
#[component]
pub fn JsonBlock(body: Option<Value>) -> impl IntoView {
    let text = body.as_ref().map(pretty_json).unwrap_or_default();
    if text.trim().is_empty() {
        return view! {
            <p class="text-xs text-muted-foreground">{t_or("alpg000.detail.no_body", "ไม่มีเนื้อหา")}</p>
        }
        .into_any();
    }
    let for_copy = text.clone();
    view! {
        <div class="space-y-2">
            <div class="flex justify-end">
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    class="h-7 px-2 text-xs"
                    on_click=Callback::new(move |_| copy_text(&for_copy))
                >
                    {t_or("alpg000.action.copy", "คัดลอก")}
                </Button>
            </div>
            <pre class="max-h-96 overflow-auto rounded-md border border-border/60 bg-muted/40 p-3 text-xs font-mono whitespace-pre-wrap break-all">{text}</pre>
        </div>
    }
    .into_any()
}

/// Key/value table of headers
#[component]
pub fn HeaderTable(headers: Option<Value>) -> impl IntoView {
    let pairs = header_pairs(&headers);
    if pairs.is_empty() {
        return view! {
            <p class="text-xs text-muted-foreground">{t_or("alpg000.detail.no_headers", "ไม่มีส่วนหัว")}</p>
        }
        .into_any();
    }
    view! {
        <div class="max-h-72 overflow-auto rounded-md border border-border/60">
            <table class="w-full text-xs">
                <tbody>
                    {pairs.into_iter().map(|(k, v)| view! {
                        <tr class="border-b border-border/40 last:border-0 align-top">
                            <td class="px-3 py-1.5 font-mono font-medium text-muted-foreground whitespace-nowrap">{k}</td>
                            <td class="px-3 py-1.5 font-mono break-all">{v}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
    .into_any()
}
