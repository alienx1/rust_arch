//! Mail delivery of the notification service: what is queued on `mail-outbox` is sent, a mail
//! server that does not answer is retried, one that can never work is not.
//! Located at `application/notification/tests/mail_delivery_test.rs`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use application_core::traits::{EventHandler, MailError, MailMessage, MailService};
use application_core::traits::{NotificationMessage};
use application_notification::features::mail_delivery::MailDeliveryHandler;
use application_notification::features::ntpg000::{Ntpg000Commands, Ntpg000Error, NotificationListDto};
use std::sync::Mutex;
use async_trait::async_trait;

/// Fails `fail_times` times with `error`, then succeeds
struct FlakyMail {
    calls: AtomicUsize,
    fail_times: usize,
    error: MailError,
}

#[async_trait]
impl MailService for FlakyMail {
    async fn send(&self, _m: MailMessage) -> Result<(), MailError> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        if n < self.fail_times { Err(self.error.clone()) } else { Ok(()) }
    }
}

fn message() -> MailMessage {
    MailMessage { to: "a@b.c".into(), subject: "s".into(), text: "t".into(), html: Some("<p>t</p>".into()), requested_by: None }
}

fn handler(fail_times: usize, error: MailError) -> (MailDeliveryHandler, Arc<FlakyMail>) {
    let mail = Arc::new(FlakyMail { calls: AtomicUsize::new(0), fail_times, error });
    (MailDeliveryHandler::new(mail.clone()).with_retry_delays(vec![std::time::Duration::ZERO; 2]), mail)
}

#[tokio::test]
async fn test_a_queued_message_is_sent() {
    let (h, mail) = handler(0, MailError::Delivery("x".into()));
    h.handle(Some("a@b.c".into()), serde_json::to_vec(&message()).unwrap()).await;
    assert_eq!(mail.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_a_mail_server_that_does_not_answer_is_retried() {
    let (h, mail) = handler(2, MailError::Delivery("timeout".into()));
    assert!(h.deliver(message()).await.is_ok(), "the third attempt goes through");
    assert_eq!(mail.calls.load(Ordering::SeqCst), 3);

    let (h, mail) = handler(10, MailError::Delivery("down".into()));
    assert!(matches!(h.deliver(message()).await, Err(MailError::Delivery(_))));
    assert_eq!(mail.calls.load(Ordering::SeqCst), 3, "three attempts, then it gives up");
}

#[tokio::test]
async fn test_a_mail_that_can_never_be_sent_is_not_retried() {
    for error in [MailError::NotConfigured, MailError::BadAddress("x".into())] {
        let (h, mail) = handler(10, error);
        assert!(h.deliver(message()).await.is_err());
        assert_eq!(mail.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn test_an_unreadable_message_is_skipped() {
    let (h, mail) = handler(0, MailError::NotConfigured);
    h.handle(None, b"not json".to_vec()).await;
    assert_eq!(mail.calls.load(Ordering::SeqCst), 0);
}

/// Collects what would land in the bell
#[derive(Default)]
struct Bell(Mutex<Vec<NotificationMessage>>);

#[async_trait]
impl Ntpg000Commands for Bell {
    async fn ingest(&self, message: NotificationMessage) {
        self.0.lock().unwrap().push(message);
    }
    async fn push_live(&self, _message: NotificationMessage) {}
    async fn list(&self, _u: &str, _l: Option<u64>) -> Result<NotificationListDto, Ntpg000Error> {
        unreachable!()
    }
    async fn mark_read(&self, _u: &str, _id: &str) -> Result<(), Ntpg000Error> {
        unreachable!()
    }
    async fn mark_all_read(&self, _u: &str) -> Result<(), Ntpg000Error> {
        unreachable!()
    }
}

#[tokio::test]
async fn test_the_person_who_asked_is_told_when_the_mail_cannot_be_delivered() {
    let bell = Arc::new(Bell::default());
    let (h, _mail) = handler(10, MailError::Delivery("refused".into()));
    let h = h.with_failure_notices(bell.clone());

    let mut asked = message();
    asked.requested_by = Some("admin".into());
    h.handle(None, serde_json::to_vec(&asked).unwrap()).await;
    let told = bell.0.lock().unwrap().clone();
    assert_eq!(told.len(), 1);
    assert_eq!(told[0].recipient, "admin");
    assert_eq!(told[0].notification.kind, "mail_failed");
    assert_eq!(told[0].notification.data["to"], "a***@b.c", "the address is not spelled out");
    assert!(told[0].notification.data["reason"].as_str().unwrap().contains("refused"));

    // Nobody asked (the holder signing in again): nobody to tell
    h.handle(None, serde_json::to_vec(&message()).unwrap()).await;
    assert_eq!(bell.0.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn test_a_delivered_mail_tells_nobody() {
    let bell = Arc::new(Bell::default());
    let (h, _mail) = handler(0, MailError::Delivery("x".into()));
    let h = h.with_failure_notices(bell.clone());
    let mut asked = message();
    asked.requested_by = Some("admin".into());
    h.handle(None, serde_json::to_vec(&asked).unwrap()).await;
    assert!(bell.0.lock().unwrap().is_empty());
}
