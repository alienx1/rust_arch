//! Configuration of the Quickwit audit adapter, read from the environment.

/// Default Quickwit REST endpoint (local dev)
pub const DEFAULT_QUICKWIT_URL: &str = "http://localhost:7280";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuickwitConfig {
    /// Base URL without a trailing slash
    pub url: String,
    /// `false` => the caller should use `NoopAuditLog` instead of starting the adapter
    pub enabled: bool,
}

impl QuickwitConfig {
    /// `QUICKWIT_URL` (default `http://localhost:7280`), `AUDIT_LOG_ENABLED` (default `true`)
    pub fn from_env() -> Self {
        Self::from_values(
            std::env::var("QUICKWIT_URL").ok().as_deref(),
            std::env::var("AUDIT_LOG_ENABLED").ok().as_deref(),
        )
    }

    /// Pure form of [`Self::from_env`]; `enabled` is false only for `false` / `0` / `no` / `off`
    pub fn from_values(url: Option<&str>, enabled: Option<&str>) -> Self {
        let url = url
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .unwrap_or(DEFAULT_QUICKWIT_URL)
            .trim_end_matches('/')
            .to_string();
        let enabled = !matches!(
            enabled.map(|v| v.trim().to_ascii_lowercase()).as_deref(),
            Some("false" | "0" | "no" | "off")
        );
        Self { url, enabled }
    }
}
