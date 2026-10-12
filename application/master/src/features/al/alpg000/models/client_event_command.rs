//! ALPG000 client-reported event

use serde::{Deserialize, Serialize};

/// One thing the web app reports: `kind` is `page_view`, `click` or `action`
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ClientEventCommand {
    pub kind: String,
    /// Route the user was on
    #[serde(default)]
    pub page: Option<String>,
    /// What was pressed
    #[serde(default)]
    pub element: Option<String>,
    /// Visible text of the element
    #[serde(default)]
    pub label: Option<String>,
    /// Process name, row id ...
    #[serde(default)]
    pub detail: Option<String>,
}
