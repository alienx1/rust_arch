//! ALPG000 client-event rules: which events are accepted and how they are completed

use application_core::traits::{AuditEvent, audit_kind};
use chrono::{SecondsFormat, Utc};
use uuid::Uuid;

use crate::features::al::alpg000::errors::Alpg000Error;
use crate::features::al::alpg000::models::{
    ClientActor, ClientEventCommand, MAX_CLIENT_EVENTS, MAX_DETAIL_LEN, MAX_ELEMENT_LEN,
    MAX_LABEL_LEN, MAX_PAGE_FIELD_LEN,
};

/// Trimmed, cut at `max` characters; blank -> `None`
fn cap(value: Option<String>, max: usize) -> Option<String> {
    let v = value?;
    let v = v.trim();
    (!v.is_empty()).then(|| v.chars().take(max).collect())
}

/// Builds the stored events for `actor`: more than 50 events is a `Validation` error; events of a
/// kind other than `page_view` / `click` / `action` (never `api`) are dropped. The server fills
/// `event_id` (UUIDv7), `timestamp` (UTC now) and the actor fields; free fields are capped.
pub fn normalize_client_events(
    actor: &ClientActor,
    events: Vec<ClientEventCommand>,
) -> Result<Vec<AuditEvent>, Alpg000Error> {
    if events.len() > MAX_CLIENT_EVENTS {
        return Err(Alpg000Error::Validation(format!(
            "At most {MAX_CLIENT_EVENTS} events per request"
        )));
    }
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    Ok(events
        .into_iter()
        .filter_map(|e| {
            let kind = e.kind.trim().to_lowercase();
            if ![audit_kind::PAGE_VIEW, audit_kind::CLICK, audit_kind::ACTION].contains(&kind.as_str()) {
                return None;
            }
            Some(AuditEvent {
                event_id: Uuid::now_v7().to_string(),
                timestamp: now.clone(),
                kind,
                user_id: actor.user_id.clone(),
                username: actor.username.clone(),
                email: actor.email.clone(),
                session_id: actor.session_id.clone(),
                ip: actor.ip.clone(),
                user_agent: actor.user_agent.clone(),
                language: actor.language.clone(),
                page: cap(e.page, MAX_PAGE_FIELD_LEN),
                element: cap(e.element, MAX_ELEMENT_LEN),
                label: cap(e.label, MAX_LABEL_LEN),
                detail: cap(e.detail, MAX_DETAIL_LEN),
                ..AuditEvent::default()
            })
        })
        .collect())
}
