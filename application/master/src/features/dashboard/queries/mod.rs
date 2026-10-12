pub mod dashboard_queries;
pub mod get_dashboard_stats;

pub use dashboard_queries::DbDashboardQueries;
pub use get_dashboard_stats::GetDashboardStatsQuery;

use crate::features::dashboard::errors::DashboardError;
use crate::features::dashboard::models::DashboardSummary;
use async_trait::async_trait;

#[async_trait]
pub trait DashboardQueries: Send + Sync {
    /// Retrieve real aggregated dashboard statistics and operational data.
    async fn fetch_summary(&self) -> Result<DashboardSummary, DashboardError>;
}
