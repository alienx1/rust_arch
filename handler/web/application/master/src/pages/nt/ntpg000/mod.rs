//! NTPG000 Page Module (Notifications: the bell in the topbar and the full list; Clean UI
//! Pattern). Notifications come from the notification service (`/api/v1/notifications`): any
//! service publishes them, this feature lists them, counts the unread ones and receives pushes.

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::Ntpg000Screen as Ntpg000Page;
