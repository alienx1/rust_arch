//! One document-level, capture-phase `click` listener that records buttons and links.
//!
//! Never records text inputs or any entered value: only the element identity and its visible
//! label (`data-audit-label`, `aria-label`, `title`, or the trimmed text, max 80 chars).

use std::cell::Cell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::audit::audit_tracker::track_click;
use crate::routes::ALPG000;

const TARGETS: &str = "button, a[href], [role=button], [data-audit]";
const MAX_LABEL: usize = 80;

thread_local! {
    static INSTALLED: Cell<bool> = const { Cell::new(false) };
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn cut(text: &str) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    one_line.chars().take(MAX_LABEL).collect()
}

/// `button#save`, `a[/db/dbpg004]`, `div[row-menu]` or the bare tag.
fn element_name(el: &web_sys::Element) -> String {
    let tag = el.tag_name().to_lowercase();
    if let Some(id) = non_blank(el.get_attribute("id")) {
        return format!("{tag}#{id}");
    }
    if let Some(mark) = non_blank(el.get_attribute("data-audit")) {
        return format!("{tag}[{mark}]");
    }
    if tag == "a" {
        if let Some(href) = non_blank(el.get_attribute("href")) {
            return format!("a[{href}]");
        }
    }
    match non_blank(el.get_attribute("role")) {
        Some(role) => format!("{tag}[role={role}]"),
        None => tag,
    }
}

fn label_of(el: &web_sys::Element) -> Option<String> {
    let text = || el.dyn_ref::<web_sys::HtmlElement>().map(|h| h.inner_text());
    non_blank(el.get_attribute("data-audit-label"))
        .or_else(|| non_blank(el.get_attribute("aria-label")))
        .or_else(|| non_blank(el.get_attribute("title")))
        .or_else(|| non_blank(text()))
        .map(|l| cut(&l))
}

fn on_click(event: web_sys::Event) {
    let Some(target) = event.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else {
        return;
    };
    let Ok(Some(el)) = target.closest(TARGETS) else { return };
    // The audit viewer's own filter buttons would only log themselves
    let on_audit_page = web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .is_some_and(|p| p.starts_with(ALPG000));
    if on_audit_page {
        return;
    }
    track_click(element_name(&el), label_of(&el));
}

/// Installs the listener (once per app run).
pub fn install() {
    if INSTALLED.with(|i| i.replace(true)) {
        return;
    }
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(on_click);
    let _ = document.add_event_listener_with_callback_and_bool(
        "click",
        listener.as_ref().unchecked_ref(),
        true,
    );
    listener.forget();
}
