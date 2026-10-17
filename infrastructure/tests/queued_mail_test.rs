//! `QueuedMailService`: the message goes to topic `mail-outbox`, keyed by recipient, and a broker
//! that is down is an error for the caller.
//! Located at `infrastructure/tests/queued_mail_test.rs`.

use std::sync::{Arc, Mutex};

use application_core::EventError;
use application_core::traits::{EventService, MAIL_TOPIC, MailError, MailMessage, MailService};
use async_trait::async_trait;
use infrastructure::QueuedMailService;

#[derive(Default)]
struct Broker {
    published: Mutex<Vec<(String, Option<String>, Vec<u8>)>>,
    down: bool,
}

#[async_trait]
impl EventService for Broker {
    async fn publish(&self, topic: &str, key: Option<&str>, payload: &[u8]) -> Result<(), EventError> {
        if self.down {
            return Err(EventError::PublishFailed { topic: topic.into(), reason: "broker down".into() });
        }
        self.published.lock().unwrap().push((topic.into(), key.map(Into::into), payload.to_vec()));
        Ok(())
    }
}

fn message(to: &str) -> MailMessage {
    MailMessage { to: to.into(), subject: "s".into(), text: "t".into(), html: None, requested_by: None }
}

#[tokio::test]
async fn test_the_message_is_queued_for_the_notification_service() {
    let broker = Arc::new(Broker::default());
    QueuedMailService::new(broker.clone()).send(message("a@b.c")).await.unwrap();
    let published = broker.published.lock().unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!((published[0].0.as_str(), published[0].1.as_deref()), (MAIL_TOPIC, Some("a@b.c")));
    let back: MailMessage = serde_json::from_slice(&published[0].2).unwrap();
    assert_eq!(back, message("a@b.c"));
}

#[tokio::test]
async fn test_a_broker_that_is_down_and_a_bad_address_are_errors() {
    let down = Arc::new(Broker { down: true, ..Default::default() });
    assert!(matches!(QueuedMailService::new(down).send(message("a@b.c")).await, Err(MailError::Delivery(_))));
    let ok = Arc::new(Broker::default());
    assert!(matches!(QueuedMailService::new(ok.clone()).send(message("nobody")).await, Err(MailError::BadAddress(_))));
    assert!(ok.published.lock().unwrap().is_empty());
}
