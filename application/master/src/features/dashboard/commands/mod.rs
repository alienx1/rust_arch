pub mod dashboard_executor;
pub mod get_dashboard;

pub use dashboard_executor::DashboardCommandExecutor;
pub use get_dashboard::{GetDashboardCommand, GetDashboardCommandHandler};

use crate::features::dashboard::errors::DashboardError;
use crate::features::dashboard::models::DashboardSummary;
use async_trait::async_trait;

#[async_trait]
pub trait DashboardCommands: Send + Sync {
    /// Retrieve real aggregated dashboard statistics and operational data.
    async fn get_dashboard_summary(&self) -> Result<DashboardSummary, DashboardError>;
}
