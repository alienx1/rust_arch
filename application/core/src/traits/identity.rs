//! Identity provisioning port: create / delete identities and issue recovery links at the
//! identity provider (Ory Kratos Admin API). Passwords are never handled by the application:
//! a new account sets its password through a one-time recovery link.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// What the identity provider needs to create an account
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewIdentity {
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

/// One-time link that lets the account holder set a password
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryLink {
    pub url: String,
    /// RFC 3339 expiry reported by the provider, when it gives one
    pub expires_at: Option<String>,
}

/// Failure of an identity-provider call
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// An identity with this email / username already exists
    Conflict(String),
    /// The provider has no such identity
    NotFound(String),
    /// The provider rejected the request or could not be reached
    Provider(String),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityError::Conflict(m) => write!(f, "identity conflict: {m}"),
            IdentityError::NotFound(m) => write!(f, "identity not found: {m}"),
            IdentityError::Provider(m) => write!(f, "identity provider error: {m}"),
        }
    }
}

impl std::error::Error for IdentityError {}

/// Port for technical adapters (Ory Kratos Admin API) to provision identities.
#[async_trait]
pub trait IdentityProvisioningPort: Send + Sync {
    /// Create an active identity without credentials; returns its provider id.
    async fn create_identity(&self, identity: NewIdentity) -> Result<String, IdentityError>;

    /// Delete an identity (used to undo a creation whose local record could not be saved).
    async fn delete_identity(&self, identity_id: &str) -> Result<(), IdentityError>;

    /// Issue a one-time recovery link for the identity behind `identifier` (email or id).
    async fn create_recovery_link(&self, identifier: &str) -> Result<RecoveryLink, IdentityError>;

    /// Provider id of the identity behind a login identifier (email), `None` when unknown.
    async fn find_identity_id(&self, identifier: &str) -> Result<Option<String>, IdentityError>;
}
