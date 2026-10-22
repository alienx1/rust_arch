//! Tells the user how their write went: one notification per create / update / delete (and per
//! failed attempt) on a program API, to the user who made it.
//!
//! The notice goes through the notification port (Redpanda -> api-notification) and reaches the
//! user's open sessions over the SSE stream of the bell, which shows it as a toast, with no
//! per-page code. A success is **live only** (`notify_live`: pushed, never stored or indexed: the
//! change itself is in the audit trail already); a failure is stored like any notification, so
//! the user can find what went wrong later.
//! Nothing here can fail or slow a request: the notice is queued in a spawned task.

use std::sync::Arc;

use application_core::traits::{Notification, NotificationService, notification_kind};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::change_target::{ChangeKind, ChangeNotice, describe_change};
use crate::middleware::auth::kratos::AuthenticatedUser;

#[derive(Clone)]
pub struct ChangeNoticeState {
    /// `None` (no broker client): the layer lets everything through untouched
    pub notifier: Option<Arc<dyn NotificationService>>,
}

impl ChangeNoticeState {
    pub fn new(notifier: Option<Arc<dyn NotificationService>>) -> Self {
        Self { notifier }
    }
}

/// The notification for `notice` (the wording is Thai; the web app translates by `kind` and `data`)
fn notification_for(notice: &ChangeNotice) -> Notification {
    let (kind, title) = match notice.kind {
        ChangeKind::Created => (notification_kind::DATA_CREATED, "บันทึกข้อมูลใหม่สำเร็จ"),
        ChangeKind::Updated => (notification_kind::DATA_UPDATED, "แก้ไขข้อมูลสำเร็จ"),
        ChangeKind::Deleted => (notification_kind::DATA_DELETED, "ลบข้อมูลสำเร็จ"),
        ChangeKind::Failed => (notification_kind::DATA_FAILED, "บันทึกข้อมูลไม่สำเร็จ"),
    };
    let what = match notice.kind {
        ChangeKind::Created => "สร้าง",
        ChangeKind::Updated => "แก้ไข",
        ChangeKind::Deleted => "ลบ",
        ChangeKind::Failed => "ทำรายการ",
    };
    let body = if notice.kind == ChangeKind::Failed {
        format!("{what} {} ในโปรแกรม {} ไม่สำเร็จ (รหัส {})", notice.target, notice.program, notice.status)
    } else {
        format!("{what} {} ในโปรแกรม {} เรียบร้อย", notice.target, notice.program)
    };
    Notification {
        id: uuid::Uuid::now_v7().to_string(),
        kind: kind.to_string(),
        title: title.to_string(),
        body,
        data: serde_json::json!({ "program": notice.program, "target": notice.target, "status": notice.status }),
        link: Some(notice.link.clone()),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Layer: runs the request, then queues the notice for whoever made it
pub async fn change_notice(State(state): State<ChangeNoticeState>, request: Request, next: Next) -> Response {
    let Some(notifier) = state.notifier.clone() else { return next.run(request).await };
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;

    if let Some(notice) = describe_change(&method, &path, response.status().as_u16())
        // The session middleware puts the signed-in user into the response; no user, nobody to tell
        && let Some(username) = response.extensions().get::<AuthenticatedUser>().and_then(AuthenticatedUser::username)
    {
        tokio::spawn(async move {
            let notification = notification_for(&notice);
            match notice.kind {
                ChangeKind::Failed => notifier.notify(&username, notification).await,
                _ => notifier.notify_live(&username, notification).await,
            }
        });
    }
    response
}
