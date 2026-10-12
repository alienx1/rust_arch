//! List all enterprise users with live sessions

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::UserSessionOverviewDto;

impl Supg005CommandExecutor {
    pub(super) async fn list_user_sessions(&self) -> Result<Vec<UserSessionOverviewDto>, Supg005Error> {
        let mut users = self.queries.list_users_with_sessions().await?;
        self.attach_live_devices(&mut users).await;
        Ok(users)
    }
}
