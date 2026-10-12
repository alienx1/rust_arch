//! NTPG000 models: a notification as the web app lists it

mod notification_dto;
mod ntpg000_constants;

pub use domain::entity::nt_notification::Model as NotificationModel;
pub use notification_dto::{NotificationDto, NotificationListDto};
pub use ntpg000_constants::*;
