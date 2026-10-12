use serde::{Deserialize, Serialize};

/// Ory Kratos active session object returned by `/sessions/whoami`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KratosSession {
    pub id: String,
    pub active: bool,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub authenticated_at: Option<String>,
    #[serde(default)]
    pub identity: Option<KratosIdentity>,
    #[serde(default)]
    pub authenticator_assurance_level: Option<String>,
    #[serde(default)]
    pub devices: Option<Vec<KratosSessionDevice>>,
}

impl KratosSession {
    /// Helper to get user's email address from identity traits.
    pub fn email(&self) -> Option<String> {
        self.identity.as_ref().and_then(|i| i.email())
    }

    /// Helper to get user's display name from identity traits.
    pub fn name(&self) -> Option<String> {
        self.identity.as_ref().and_then(|i| i.name())
    }

    /// Helper to get user's identity ID.
    pub fn identity_id(&self) -> Option<String> {
        self.identity.as_ref().map(|i| i.id.clone())
    }
}

/// Device metadata associated with an Ory Kratos active session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KratosSessionDevice {
    pub id: String,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
}

/// User identity details extracted from Kratos session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KratosIdentity {
    pub id: String,
    #[serde(default)]
    pub schema_id: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub traits: serde_json::Value,
}

impl KratosIdentity {
    /// Helper to get user's email address from identity traits.
    pub fn email(&self) -> Option<String> {
        self.traits
            .get("email")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
    }

    /// Helper to get user's display name from identity traits.
    pub fn name(&self) -> Option<String> {
        self.traits
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| {
                // Try traits.name.first or traits.username
                self.traits.get("username").and_then(|v| v.as_str())
            })
            .map(ToString::to_string)
    }
}

/// Self-service login flow returned by Kratos `/self-service/login/browser`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosLoginFlow {
    pub id: String,
    pub ui: KratosUiContainer,
    #[serde(default)]
    pub expires_at: Option<String>,
}

impl KratosLoginFlow {
    /// Extract the CSRF token from the UI form nodes.
    pub fn csrf_token(&self) -> Option<String> {
        self.ui.csrf_token()
    }

    /// Retrieve all top-level UI messages (e.g. general errors or instructions).
    pub fn general_messages(&self) -> Vec<String> {
        self.ui
            .messages
            .as_ref()
            .map(|msgs| msgs.iter().map(|m| m.text.clone()).collect())
            .unwrap_or_default()
    }

    /// Extract field-specific error message for a given field name (e.g. "identifier", "password").
    pub fn field_error(&self, field_name: &str) -> Option<String> {
        self.ui.nodes.iter().find_map(|node| {
            if node.attributes.name.as_deref() == Some(field_name) {
                node.messages.first().map(|m| m.text.clone())
            } else {
                None
            }
        })
    }
}

/// Self-service registration flow returned by Kratos `/self-service/registration/browser`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosRegistrationFlow {
    pub id: String,
    pub ui: KratosUiContainer,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Logout flow payload returned by Kratos `/self-service/logout/browser`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosLogoutFlow {
    pub logout_url: String,
    pub logout_token: String,
}

/// Kratos UI container enclosing form action and form fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosUiContainer {
    pub action: String,
    pub method: String,
    #[serde(default)]
    pub nodes: Vec<KratosUiNode>,
    #[serde(default)]
    pub messages: Option<Vec<KratosUiMessage>>,
}

impl KratosUiContainer {
    /// Extract the CSRF token from the container nodes.
    pub fn csrf_token(&self) -> Option<String> {
        self.nodes.iter().find_map(|node| {
            if node.attributes.name.as_deref() == Some("csrf_token") {
                node.attributes.value.as_ref().and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
    }
}

/// Form input or button node in a Kratos UI flow.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosUiNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub group: String,
    pub attributes: KratosUiNodeAttributes,
    #[serde(default)]
    pub messages: Vec<KratosUiMessage>,
}

/// Attributes for a Kratos UI form node (e.g. input name, type, value, csrf).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosUiNodeAttributes {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    pub input_type: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(rename = "node_type", default)]
    pub node_type: Option<String>,
}

/// Error or notification message generated by Kratos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KratosUiMessage {
    pub id: i64,
    pub text: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

/// Credentials payload for password-based login.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub identifier: String,
    pub password: String,
    #[serde(default = "default_password_method")]
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf_token: Option<String>,
}

fn default_password_method() -> String {
    "password".to_string()
}

impl LoginPayload {
    pub fn new(identifier: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            password: password.into(),
            method: "password".to_string(),
            csrf_token: None,
        }
    }

    pub fn with_csrf(mut self, token: impl Into<String>) -> Self {
        self.csrf_token = Some(token.into());
        self
    }
}
