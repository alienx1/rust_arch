//! Session revocation and IAM identity management port.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Active identity session as reported by the identity provider (Ory Kratos).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentitySessionInfo {
    pub session_id: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<String>,
    pub authenticated_at: Option<String>,
}

/// Port for technical adapters (e.g. Ory Kratos Admin API) to revoke active sessions.
#[async_trait]
pub trait SessionRevocationPort: Send + Sync {
    /// Invalidate a specific active session by ID.
    async fn revoke_session(&self, session_id: &str) -> Result<bool, String>;

    /// Invalidate all active sessions for an identity/user.
    async fn revoke_user_sessions(&self, user_id: &str) -> Result<bool, String>;

    /// Enable (`true`) or disable (`false`) login for an identity/user at the identity provider.
    /// Returns `Ok(false)` when the provider has no such identity.
    async fn set_user_active(&self, user_id: &str, active: bool) -> Result<bool, String>;

    /// List active sessions of an identity/user (empty when the identity is unknown).
    async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<IdentitySessionInfo>, String>;
}
