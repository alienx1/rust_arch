//! Ory Kratos Authentication module for Leptos WebAssembly applications.

pub mod client;
pub mod context;
pub mod login_page;
pub mod settings_page;
pub mod types;

pub use crate::core::errors::AuthError;
pub use client::KratosClient;
pub use context::{AuthContext, provide_auth_context, use_auth};
pub use login_page::LoginPage;
pub use settings_page::SettingsPage;
pub use types::{
    KratosIdentity, KratosLoginFlow, KratosLogoutFlow, KratosRegistrationFlow, KratosSession,
    KratosUiContainer, KratosUiMessage, KratosUiNode, KratosUiNodeAttributes, LoginPayload,
};
