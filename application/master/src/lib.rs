//! Application Master: Business logic, CQRS commands & queries, and feature domain services.

pub mod features;

pub use features::dashboard;
pub use features::localization;
pub use features::*;
