//! Tests for NTPG000 (notification service): ingest (store + push, once), the user's list,
//! unread count and mark-as-read, over in-memory ports. Located at
//! `application/notification/tests/ntpg000_test.rs`.

use application_core::traits::{
    EventHandler, Notification, NotificationMessage, NotificationPush, PushEvent,
};
use application_notification::features::ntpg000::{
    NotificationMessageHandler, NotificationModel, Ntpg000CommandExecutor, Ntpg000Commands, Ntpg000Error,
    Ntpg000Queries,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
struct MemoryNotifications {
    rows: Mutex<Vec<NotificationModel>>,
}

#[async_trait]
impl Ntpg000Queries for MemoryNotifications {
    async fn insert_if_absent(&self, row: NotificationModel) -> Result<bool, Ntpg000Error> {
        let mut rows = self.rows.lock().unwrap();
        if rows.iter().any(|r| r.id == row.id) {
            return Ok(false);
        }
        rows.push(row);
        Ok(true)
    }
    async fn list(&self, recipient: &str, limit: u64) -> Result<Vec<NotificationModel>, Ntpg000Error> {
        let mut mine: Vec<_> =
            self.rows.lock().unwrap().iter().filter(|r| r.recipient == recipient).cloned().collect();
        mine.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        mine.truncate(limit as usize);
        Ok(mine)
    }
    async fn prune(&self, recipient: &str, keep: u64) -> Result<u64, Ntpg000Error> {
        let mut rows = self.rows.lock().unwrap();
        let mut mine: Vec<_> = rows.iter().filter(|r| r.recipient == recipient).map(|r| (r.created_at, r.id)).collect();
        mine.sort_by(|a, b| b.cmp(a));
        let stale: Vec<Uuid> = mine.into_iter().skip(keep as usize).map(|(_, id)| id).collect();
        rows.retain(|r| !stale.contains(&r.id));
        Ok(stale.len() as u64)
    }
    async fn count_unread(&self, recipient: &str) -> Result<u64, Ntpg000Error> {
        Ok(self.rows.lock().unwrap().iter().filter(|r| r.recipient == recipient && r.read_at.is_none()).count() as u64)
    }
    async fn mark_read(&self, recipient: &str, id: Uuid) -> Result<bool, Ntpg000Error> {
        let mut rows = self.rows.lock().unwrap();
        Ok(rows
            .iter_mut()
            .find(|r| r.id == id && r.recipient == recipient && r.read_at.is_none())
            .map(|r| r.read_at = Some(Utc::now().fixed_offset()))
            .is_some())
    }
    async fn mark_all_read(&self, recipient: &str) -> Result<u64, Ntpg000Error> {
        let mut n = 0;
        for r in self.rows.lock().unwrap().iter_mut().filter(|r| r.recipient == recipient && r.read_at.is_none()) {
            r.read_at = Some(Utc::now().fixed_offset());
            n += 1;
        }
        Ok(n)
    }
}

#[derive(Default)]
struct RecordingPush {
    sent: Mutex<Vec<(String, PushEvent)>>,
}

#[async_trait]
impl NotificationPush for RecordingPush {
    async fn push(&self, username: &str, event: PushEvent) {
        self.sent.lock().unwrap().push((username.into(), event));
    }
}

struct Rig {
    exec: Arc<Ntpg000CommandExecutor>,
    rows: Arc<MemoryNotifications>,
    push: Arc<RecordingPush>,
}

fn rig() -> Rig {
    let rows = Arc::new(MemoryNotifications::default());
    let push = Arc::new(RecordingPush::default());
    Rig { exec: Arc::new(Ntpg000CommandExecutor::new(rows.clone(), push.clone())), rows, push }
}

fn message(to: &str, id: Uuid, at: &str) -> NotificationMessage {
    NotificationMessage {
        recipient: to.into(),
        notification: Notification {
            id: id.to_string(),
            kind: "report_ready".into(),
            title: "ready".into(),
            body: "file.csv".into(),
            data: serde_json::json!({ "job_id": "j1" }),
            link: Some("/rp/rppg000".into()),
            created_at: at.into(),
        },
    }
}

#[tokio::test]
async fn test_ingest_stores_and_pushes_to_the_recipient_once() {
    let r = rig();
    let id = Uuid::now_v7();
    r.exec.ingest(message("somchai", id, "2026-10-12T01:00:00Z")).await;
    r.exec.ingest(message("somchai", id, "2026-10-12T01:00:00Z")).await; // redelivered

    assert_eq!(r.rows.rows.lock().unwrap().len(), 1);
    let sent = r.push.sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "a redelivered message is not pushed again");
    assert_eq!(sent[0].0, "somchai");
    let PushEvent::Notification(n) = &sent[0].1 else { panic!("expected a notification") };
    assert_eq!((n.id.as_str(), n.link.as_deref()), (id.to_string().as_str(), Some("/rp/rppg000")));
}

#[tokio::test]
async fn test_ingest_skips_what_cannot_be_stored_and_drops_foreign_links() {
    let r = rig();
    let mut bad_id = message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z");
    bad_id.notification.id = "not-a-uuid".into();
    r.exec.ingest(bad_id).await;
    r.exec.ingest(message("  ", Uuid::now_v7(), "2026-10-12T01:00:00Z")).await;
    assert!(r.rows.rows.lock().unwrap().is_empty());
    assert!(r.push.sent.lock().unwrap().is_empty());

    for link in ["https://evil.example/x", "//evil.example", "javascript:alert(1)"] {
        let mut m = message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z");
        m.notification.link = Some(link.into());
        r.exec.ingest(m).await;
    }
    assert!(r.rows.rows.lock().unwrap().iter().all(|n| n.link.is_none()), "a notification never links off-site");
}

#[tokio::test]
async fn test_list_is_the_users_own_newest_first_with_the_unread_count() {
    let r = rig();
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    r.exec.ingest(message("somchai", a, "2026-10-12T01:00:00Z")).await;
    r.exec.ingest(message("somchai", b, "2026-10-12T02:00:00Z")).await;
    r.exec.ingest(message("somying", Uuid::now_v7(), "2026-10-12T03:00:00Z")).await;

    let list = r.exec.list("somchai", None).await.unwrap();
    assert_eq!(list.unread, 2);
    assert_eq!(list.items.iter().map(|n| n.id.clone()).collect::<Vec<_>>(), vec![b.to_string(), a.to_string()]);
    assert!(list.items.iter().all(|n| !n.read));
    assert_eq!(r.exec.list("somchai", Some(1)).await.unwrap().items.len(), 1);
    assert_eq!(r.exec.list("nobody", None).await.unwrap(), Default::default());
}

#[tokio::test]
async fn test_mark_read_updates_the_count_and_tells_the_other_sessions() {
    let r = rig();
    let id = Uuid::now_v7();
    r.exec.ingest(message("somchai", id, "2026-10-12T01:00:00Z")).await;
    r.push.sent.lock().unwrap().clear();

    r.exec.mark_read("somchai", &id.to_string()).await.unwrap();
    let list = r.exec.list("somchai", None).await.unwrap();
    assert_eq!((list.unread, list.items[0].read), (0, true));
    assert_eq!(r.push.sent.lock().unwrap().as_slice(), &[("somchai".to_string(), PushEvent::Changed)]);

    // Reading again is fine and silent
    r.exec.mark_read("somchai", &id.to_string()).await.unwrap();
    assert_eq!(r.push.sent.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_somebody_elses_or_unknown_notification_is_not_found() {
    let r = rig();
    let id = Uuid::now_v7();
    r.exec.ingest(message("somchai", id, "2026-10-12T01:00:00Z")).await;
    for (user, target) in [("somying", id.to_string()), ("somchai", Uuid::now_v7().to_string()), ("somchai", "x".into())] {
        assert!(matches!(r.exec.mark_read(user, &target).await, Err(Ntpg000Error::NotFound(_))), "{user} {target}");
    }
    assert_eq!(r.exec.list("somchai", None).await.unwrap().unread, 1, "nothing changed");
}

#[tokio::test]
async fn test_mark_all_read_only_touches_the_user_and_pushes_only_when_something_changed() {
    let r = rig();
    r.exec.ingest(message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z")).await;
    r.exec.ingest(message("somchai", Uuid::now_v7(), "2026-10-12T02:00:00Z")).await;
    r.exec.ingest(message("somying", Uuid::now_v7(), "2026-10-12T03:00:00Z")).await;
    r.push.sent.lock().unwrap().clear();

    r.exec.mark_all_read("somchai").await.unwrap();
    assert_eq!(r.exec.list("somchai", None).await.unwrap().unread, 0);
    assert_eq!(r.exec.list("somying", None).await.unwrap().unread, 1);
    assert_eq!(r.push.sent.lock().unwrap().len(), 1);
    r.exec.mark_all_read("somchai").await.unwrap();
    assert_eq!(r.push.sent.lock().unwrap().len(), 1, "nothing to mark, nothing to tell");
}

#[tokio::test]
async fn test_handler_ingests_readable_messages_and_skips_garbage() {
    let r = rig();
    let handler = NotificationMessageHandler::new(r.exec.clone());
    handler.handle(None, b"not json".to_vec()).await;
    assert!(r.rows.rows.lock().unwrap().is_empty());
    let m = message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z");
    handler.handle(Some("somchai".into()), serde_json::to_vec(&m).unwrap()).await;
    assert_eq!(r.rows.rows.lock().unwrap().len(), 1);
    // the wire form: the notification fields sit next to `recipient`
    let wire: serde_json::Value = serde_json::to_value(&m).unwrap();
    assert_eq!(wire["recipient"], "somchai");
    assert_eq!(wire["kind"], "report_ready");
}

#[tokio::test]
async fn test_a_user_keeps_at_most_300_notifications_oldest_go_first() {
    let r = rig();
    let mut ids = Vec::new();
    for i in 0..305u32 {
        let id = Uuid::now_v7();
        ids.push(id);
        let at = format!("2026-10-12T{:02}:{:02}:{:02}Z", i / 3600, (i / 60) % 60, i % 60);
        r.exec.ingest(message("somchai", id, &at)).await;
    }
    r.exec.ingest(message("somsri", Uuid::now_v7(), "2026-10-12T00:00:00Z")).await;

    let rows = r.rows.rows.lock().unwrap();
    let mine: Vec<_> = rows.iter().filter(|n| n.recipient == "somchai").collect();
    assert_eq!(mine.len(), 300, "capped per user");
    assert!(ids[..5].iter().all(|old| mine.iter().all(|n| n.id != *old)), "the 5 oldest were removed");
    assert!(mine.iter().any(|n| n.id == ids[304]), "the newest is kept");
    assert_eq!(rows.iter().filter(|n| n.recipient == "somsri").count(), 1, "another user is untouched");
}

#[tokio::test]
async fn test_a_fresh_notification_stored_by_another_instance_still_reaches_this_instances_streams() {
    // Two instances read every message (each has its own consumer group); whichever stores it first
    // must not leave the other's open streams without it. The row is stored once.
    let r = rig();
    let id = Uuid::now_v7();
    let now = chrono::Utc::now().to_rfc3339();
    r.rows.insert_if_absent(row_of(&message("somchai", id, &now))).await.unwrap(); // the other instance
    r.exec.ingest(message("somchai", id, &now)).await;
    assert_eq!(r.push.sent.lock().unwrap().len(), 1, "pushed to this instance's streams");
    assert_eq!(r.rows.list("somchai", 10).await.unwrap().len(), 1, "stored once");

    // An old duplicate (a replay after a restart) does not ring again
    let old = Uuid::now_v7();
    let long_ago = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    r.rows.insert_if_absent(row_of(&message("somchai", old, &long_ago))).await.unwrap();
    r.exec.ingest(message("somchai", old, &long_ago)).await;
    assert_eq!(r.push.sent.lock().unwrap().len(), 1, "nothing more for the replayed one");
}

fn row_of(m: &NotificationMessage) -> application_notification::features::ntpg000::NotificationModel {
    use chrono::DateTime;
    application_notification::features::ntpg000::NotificationModel {
        id: Uuid::parse_str(&m.notification.id).unwrap(),
        recipient: m.recipient.clone(),
        kind: m.notification.kind.clone(),
        title: m.notification.title.clone(),
        body: m.notification.body.clone(),
        data: m.notification.data.clone(),
        link: m.notification.link.clone(),
        created_at: DateTime::parse_from_rfc3339(&m.notification.created_at).unwrap(),
        read_at: None,
        row_version: 1,
    }
}

#[tokio::test]
async fn test_a_live_notice_is_pushed_and_nothing_is_stored() {
    let r = rig();
    let id = Uuid::now_v7();
    r.exec.push_live(message("somchai", id, "2026-10-12T01:00:00Z")).await;
    r.exec.push_live(message("somchai", id, "2026-10-12T01:00:00Z")).await; // nothing to dedupe: it is not kept
    let sent = r.push.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 2);
    assert!(matches!(&sent[0], (who, PushEvent::Live(n)) if who == "somchai" && n.id == id.to_string()), "{sent:?}");
    let list = r.exec.list("somchai", Some(10)).await.unwrap();
    assert_eq!((list.items.len(), list.unread), (0, 0), "no row, no unread count");
}

#[tokio::test]
async fn test_a_live_notice_without_a_recipient_is_dropped_and_a_foreign_link_removed() {
    let r = rig();
    r.exec.push_live(message("  ", Uuid::now_v7(), "2026-10-12T01:00:00Z")).await;
    assert!(r.push.sent.lock().unwrap().is_empty());

    let mut m = message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z");
    m.notification.link = Some("https://evil.example/x".into());
    r.exec.push_live(m).await;
    let sent = r.push.sent.lock().unwrap().clone();
    assert!(matches!(&sent[0].1, PushEvent::Live(n) if n.link.is_none()), "{sent:?}");
}

#[tokio::test]
async fn test_the_live_handler_pushes_readable_messages_and_skips_garbage() {
    use application_core::traits::EventHandler;
    use application_notification::features::ntpg000::LiveNotificationHandler;
    let r = rig();
    let handler = LiveNotificationHandler::new(r.exec.clone());
    handler.handle(None, serde_json::to_vec(&message("somchai", Uuid::now_v7(), "2026-10-12T01:00:00Z")).unwrap()).await;
    handler.handle(None, b"not json".to_vec()).await;
    assert_eq!(r.push.sent.lock().unwrap().len(), 1);
}
