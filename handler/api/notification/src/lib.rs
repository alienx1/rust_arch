//! Notification API Service (api-notification): consumes `notifications` from Redpanda, stores
//! them and pushes them to the recipients' open web sessions; serves the list, the unread count
//! and "mark as read".

pub mod connection;
pub mod routers;

pub use connection::NotificationConnection;
pub use routers::build_app_router;
