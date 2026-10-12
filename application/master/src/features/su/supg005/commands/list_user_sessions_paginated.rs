//! List enterprise users with live sessions, paginated

use crate::features::su::supg005::commands::Supg005CommandExecutor;
use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::models::{PaginatedResult, UserPaginationQuery, UserSessionOverviewDto};

impl Supg005CommandExecutor {
    pub(super) async fn list_user_sessions_page(
        &self,
        query: UserPaginationQuery,
    ) -> Result<PaginatedResult<UserSessionOverviewDto>, Supg005Error> {
        let mut page = self.queries.list_users_paginated(query).await?;
        self.attach_live_devices(&mut page.items).await;
        Ok(page)
    }
}
