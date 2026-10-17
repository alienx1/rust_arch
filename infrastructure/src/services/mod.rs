//! Infrastructure service implementations for Core traits.

pub mod cache;
pub mod content;
pub mod event;
pub mod mail;
pub mod notification;
pub mod quickwit;
pub mod sql_context;

pub use cache::ValkeyCacheService;
pub use content::{S3ContentService, s3_client_from_env};
pub use mail::{QueuedMailService, SmtpMailService, ConfiguredMailService};
pub use event::{RedpandaEventConsumer, RedpandaEventService};
pub use notification::{RedpandaNotificationPublisher, SseNotificationHub};
pub use quickwit::{QuickwitAuditLog, QuickwitConfig, spawn_data_change_shipper};
pub use sql_context::PgEdgeSqlContext;
pub use sql_context::audit_actor::{AuditActor, current_actor, track_actor, with_actor};
