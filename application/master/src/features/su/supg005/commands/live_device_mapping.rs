//! Maps identity-provider live sessions onto user overview devices

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::models::{ActiveDeviceDto, UserSessionOverviewDto};

impl Supg005CommandExecutor {
    /// Replace directory/seed devices with the live sessions held by the identity provider.
    /// Users whose lookup fails keep an empty list rather than showing stale/fake devices.
    pub(super) async fn attach_live_devices(&self, users: &mut [UserSessionOverviewDto]) {
        let Some(ref rev) = self.revocation else { return };
        for user in users.iter_mut() {
            user.devices = match rev.list_user_sessions(&user.username).await {
                Ok(sessions) => sessions
                    .into_iter()
                    .map(|s| ActiveDeviceDto {
                        id: s.session_id,
                        user_id: user.user_id.clone(),
                        device_name: s.user_agent.unwrap_or_else(|| "Unknown device".to_string()),
                        ip_address: s.ip_address.unwrap_or_else(|| "-".to_string()),
                        location: s.location.unwrap_or_else(|| "-".to_string()),
                        last_active: "Active".to_string(),
                        authenticated_at: s.authenticated_at.unwrap_or_else(|| "-".to_string()),
                        is_current: false,
                    })
                    .collect(),
                Err(e) => {
                    tracing::warn!("Could not list sessions for user {}: {e}", user.user_id);
                    Vec::new()
                }
            };
        }
    }
}
