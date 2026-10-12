//! NTPG000 HTTP service: the signed-in user's notifications (`/api/v1/notifications`, served by
//! api-notification) and the server-sent-event stream of pushes.

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_package::core::errors::HttpError;
use web_package::core::requestor::HttpClient;

use crate::pages::nt::ntpg000::models::NotificationListDto;
use crate::routes::percent_encode;

/// API base path of the notification service
pub const NOTIFICATIONS_API: &str = "/api/v1/notifications";

#[derive(Clone, Debug)]
pub struct Ntpg000Service {
    http: HttpClient,
}

impl Default for Ntpg000Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Ntpg000Service {
    pub fn new() -> Self {
        Self { http: HttpClient::default() }
    }

    /// `GET /items?limit=`
    pub async fn fetch_list(&self, limit: u64) -> Result<NotificationListDto, String> {
        let res = self
            .http
            .get(&format!("{NOTIFICATIONS_API}/items?limit={limit}"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        res.auto::<NotificationListDto>().map_err(|e| e.to_string())
    }

    /// `POST /{id}/read`
    pub async fn mark_read(&self, id: &str) -> Result<(), HttpError> {
        self.http.post(&format!("{NOTIFICATIONS_API}/{}/read", percent_encode(id))).send().await.map(|_| ())
    }

    /// `POST /read-all`
    pub async fn mark_all_read(&self) -> Result<(), HttpError> {
        self.http.post(&format!("{NOTIFICATIONS_API}/read-all")).send().await.map(|_| ())
    }

    /// Opens the push stream (`GET /stream`); `on_event` runs for every `notification` (new one)
    /// and `changed` (read elsewhere) event, so the caller reloads the list. The browser
    /// reconnects by itself after a drop. Dropping the returned stream closes it.
    pub fn open_stream(on_event: impl Fn() + Clone + 'static) -> Option<NotificationStream> {
        let source = web_sys::EventSource::new(&format!("{NOTIFICATIONS_API}/stream")).ok()?;
        let mut handlers = Vec::new();
        for name in ["notification", "changed"] {
            let on_event = on_event.clone();
            let handler = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |_: web_sys::MessageEvent| on_event());
            source.add_event_listener_with_callback(name, handler.as_ref().unchecked_ref()).ok()?;
            handlers.push(handler);
        }
        Some(NotificationStream { source, _handlers: handlers })
    }
}

/// An open SSE connection; closed when dropped
pub struct NotificationStream {
    source: web_sys::EventSource,
    _handlers: Vec<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

impl Drop for NotificationStream {
    fn drop(&mut self) {
        self.source.close();
    }
}
