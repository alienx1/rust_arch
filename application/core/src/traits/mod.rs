//! Application core traits (Dependency Inversion contracts).

pub mod audit_log;
pub mod cache;
pub mod memory_cache;
pub mod read_cache;
pub mod content;
pub mod error_templates;
pub mod event;
pub mod event_consumer;
pub mod identity;
pub mod notification;
pub mod report_job;
pub mod session;
pub mod sql_context;

pub use audit_log::{AUDIT_INDEX, EVENT_NOTIFICATION_INDEX, EVENT_REPORT_INDEX, AuditError, AuditExportChunk, AuditEvent, AuditLogPort, AuditPage, NoopAuditLog, AuditSearch, audit_kind};
pub use cache::CacheService;
pub use memory_cache::InMemoryCache;
pub use read_cache::{CacheCounters, CacheRefresher, LOG_TARGET as CACHE_LOG_TARGET, ReadCache, ReadCaches, cache_ns, cache_ttl, spawn_stats_logger};
pub use content::ContentService;
pub use error_templates::{ErrorTemplateError, ErrorTemplateSource};
pub use event::EventService;
pub use event_consumer::{EventConsumer, EventHandler};
pub use notification::{
    NOTIFICATIONS_TOPIC, NOTIFICATION_SERVICE_GROUP, Notification, NotificationMessage, NotificationPush,
    NotificationService, PushEvent, notification_kind,
};
pub use report_job::{REPORT_REQUESTS_TOPIC, REPORT_WORKER_GROUP, ReportJobMessage, report_kind};
pub use identity::{IdentityError, IdentityProvisioningPort, NewIdentity, RecoveryLink};
pub use session::{IdentitySessionInfo, SessionRevocationPort};
pub use sql_context::SqlContext;
