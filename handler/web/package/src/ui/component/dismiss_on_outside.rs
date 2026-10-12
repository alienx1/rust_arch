//! Closes a dropdown when the user clicks anywhere outside it or presses Escape.
//!
//! A full-screen "click-away" layer does not work inside the sticky topbar: its `backdrop-blur`
//! makes `position: fixed` relative to the header, so the layer would only cover the header and
//! clicks on the page would never reach it. A window-level listener has no such limit.

use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// While `open`, a click outside `root` or the Escape key calls `close`. Call it once in the
/// component that owns the dropdown, with `node_ref=root` on the element wrapping the trigger and
/// the panel (so a click on the trigger itself is not "outside").
pub fn dismiss_on_outside(root: NodeRef<html::Div>, open: impl Fn() -> bool + Clone + 'static, close: impl Fn() + Clone + 'static) {
    let (open_on_click, close_on_click) = (open.clone(), close.clone());
    let click = window_event_listener(ev::click, move |event| {
        if !open_on_click() {
            return;
        }
        let inside = event
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
            .zip(root.get_untracked())
            .is_some_and(|(target, root)| root.contains(Some(&target)));
        if !inside {
            close_on_click();
        }
    });
    let key = window_event_listener(ev::keydown, move |event| {
        if open() && event.key() == "Escape" {
            close();
        }
    });
    on_cleanup(move || {
        click.remove();
        key.remove();
    });
}
