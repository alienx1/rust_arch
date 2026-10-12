use crate::features::dashboard::commands::DashboardCommands;
use crate::features::dashboard::errors::DashboardError;
use crate::features::dashboard::models::DashboardSummary;
use std::sync::Arc;

/// Command DTO for requesting dashboard summary.
#[derive(Clone, Debug, Default)]
pub struct GetDashboardCommand;

/// Command Handler executing `GetDashboardCommand`.
pub struct GetDashboardCommandHandler {
    commands: Arc<dyn DashboardCommands>,
}

impl GetDashboardCommandHandler {
    pub fn new(commands: Arc<dyn DashboardCommands>) -> Self {
        Self { commands }
    }

    pub async fn handle(
        &self,
        _cmd: GetDashboardCommand,
    ) -> Result<DashboardSummary, DashboardError> {
        self.commands.get_dashboard_summary().await
    }
}
