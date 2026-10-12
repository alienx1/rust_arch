//! Components module for IAM Session page

pub mod session_devices_card;
pub mod session_identity_card;
pub mod session_permissions_table;
pub mod session_raw_inspector;
pub mod session_scope_card;
pub mod session_security_card;
pub mod session_stats_kpis;
pub mod session_users_table;

pub use session_devices_card::SessionDevicesCard;
pub use session_identity_card::SessionIdentityCard;
pub use session_permissions_table::SessionPermissionsTable;
pub use session_raw_inspector::SessionRawInspector;
pub use session_scope_card::SessionScopeCard;
pub use session_security_card::SessionSecurityCard;
pub use session_stats_kpis::SessionStatsKpis;
pub use session_users_table::SessionUsersTable;
