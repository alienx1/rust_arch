//! Event queue + batched flush of the access audit.
//!
//! Events are queued in memory and sent in one POST every ~3 s, as soon as 20 are queued, and
//! (best effort) when the page is hidden. Failures are ignored; the queue is capped.

use std::cell::{Cell, RefCell};

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_package::core::requestor::HttpClient;

use crate::audit::audit_event::{
    AuditEventBatchDto, AuditEventDto, KIND_ACTION, KIND_CLICK, KIND_PAGE_VIEW,
};

const ENDPOINT: &str = "/api/v1/audit/events";
const FLUSH_INTERVAL_MS: i32 = 3_000;
const FLUSH_AT: usize = 20;
const MAX_PER_REQUEST: usize = 50;
const MAX_QUEUED: usize = 200;

thread_local! {
    static QUEUE: RefCell<Vec<AuditEventDto>> = const { RefCell::new(Vec::new()) };
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static FLUSHING: Cell<bool> = const { Cell::new(false) };
    static STARTED: Cell<bool> = const { Cell::new(false) };
    static LAST_PAGE_VIEW: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Tracking is on only while the user is signed in; turning it off drops the queue.
pub fn set_enabled(enabled: bool) {
    ENABLED.with(|e| e.set(enabled));
    if !enabled {
        QUEUE.with(|q| q.borrow_mut().clear());
        LAST_PAGE_VIEW.with(|p| p.borrow_mut().clear());
    }
}

fn current_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default()
}

fn enqueue(event: AuditEventDto) {
    if !ENABLED.with(Cell::get) {
        return;
    }
    let queued = QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        if q.len() >= MAX_QUEUED {
            return None;
        }
        q.push(event);
        Some(q.len())
    });
    if queued.is_some_and(|n| n >= FLUSH_AT) {
        flush();
    }
}

/// Records a visit of `path`; the same path twice in a row (e.g. a re-mount) counts once.
pub fn track_page_view(path: &str) {
    let repeated = LAST_PAGE_VIEW.with(|p| {
        let mut last = p.borrow_mut();
        let same = *last == path;
        if !same {
            *last = path.to_string();
        }
        same
    });
    if repeated {
        return;
    }
    enqueue(AuditEventDto {
        kind: KIND_PAGE_VIEW,
        page: path.to_string(),
        element: None,
        label: None,
        detail: None,
    });
}

/// Records a click on a button or link (see `audit_click_listener`).
pub fn track_click(element: String, label: Option<String>) {
    enqueue(AuditEventDto {
        kind: KIND_CLICK,
        page: current_path(),
        element: Some(element),
        label,
        detail: None,
    });
}

/// Records a finished process (save, delete, ...), e.g. `track_action("DBPG004.save", &code)`.
pub fn track_action(name: &str, detail: &str) {
    enqueue(AuditEventDto {
        kind: KIND_ACTION,
        page: current_path(),
        element: None,
        label: Some(name.to_string()),
        detail: (!detail.is_empty()).then(|| detail.to_string()),
    });
}

fn take_batch() -> Vec<AuditEventDto> {
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        let n = q.len().min(MAX_PER_REQUEST);
        q.drain(..n).collect()
    })
}

/// Sends up to one request's worth of queued events; errors are dropped silently.
pub fn flush() {
    if FLUSHING.with(Cell::get) {
        return;
    }
    let events = take_batch();
    if events.is_empty() {
        return;
    }
    FLUSHING.with(|f| f.set(true));
    wasm_bindgen_futures::spawn_local(async move {
        let body = AuditEventBatchDto { events };
        if let Ok(req) = HttpClient::default().post(ENDPOINT).cbor(&body) {
            let _ = req.send().await;
        }
        FLUSHING.with(|f| f.set(false));
    });
}

/// Last chance when the page goes away: `sendBeacon` with a JSON body (best effort).
fn flush_on_hide() {
    let events = take_batch();
    if events.is_empty() {
        return;
    }
    let Some(window) = web_sys::window() else { return };
    let Ok(json) = serde_json::to_string(&AuditEventBatchDto { events }) else { return };
    let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(&json));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("application/json");
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) {
        let _ = window.navigator().send_beacon_with_opt_blob(ENDPOINT, Some(&blob));
    }
}

/// Starts the periodic flush and the `pagehide` flush, once.
pub fn start() {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let Some(window) = web_sys::window() else { return };

    let tick = Closure::<dyn FnMut()>::new(flush);
    let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
        tick.as_ref().unchecked_ref(),
        FLUSH_INTERVAL_MS,
    );
    tick.forget();

    let hide = Closure::<dyn FnMut()>::new(flush_on_hide);
    let _ = window.add_event_listener_with_callback("pagehide", hide.as_ref().unchecked_ref());
    hide.forget();
}
