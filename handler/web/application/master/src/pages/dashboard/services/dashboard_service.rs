//! Dashboard API Service

use crate::pages::dashboard::models::DashboardSummary;
use web_package::core::errors::HttpError;
use web_package::core::requestor::HttpClient;

/// Service client for dashboard endpoints.
#[derive(Clone, Debug, Default)]
pub struct DashboardService {
    client: HttpClient,
}

impl DashboardService {
    pub fn new() -> Self {
        Self {
            client: HttpClient::default(),
        }
    }

    /// Retrieve live dashboard statistics from `/api/v1/master/dashboard/summary` (or `/api/v1/dashboard/summary`).
    /// Automatically decodes binary CBOR if returned, or falls back to JSON.
    pub async fn fetch_summary(&self) -> Result<DashboardSummary, HttpError> {
        let resp = match self.client.get("/api/v1/master/dashboard/summary").send().await {
            Ok(r) => Ok(r),
            Err(_) => self.client.get("/api/v1/dashboard/summary").send().await,
        }?;
        let summary: DashboardSummary = resp.auto()?;
        Ok(summary)
    }
}
