//! ALPG001 list action

use chrono::Utc;

use super::alpg001_executor::Alpg001CommandExecutor;
use super::filter_rules::normalize_process_filter;
use crate::features::al::alpg001::errors::Alpg001Error;
use crate::features::al::alpg001::models::{ProcessDto, ProcessFilterQuery, ProcessPageDto};

impl Alpg001CommandExecutor {
    pub(super) async fn list_processes_action(
        &self,
        filter: ProcessFilterQuery,
    ) -> Result<ProcessPageDto, Alpg001Error> {
        let search = normalize_process_filter(filter)?;
        let (rows, total, counts) = self.queries.search(&search).await?;
        let now = Utc::now();
        Ok(ProcessPageDto {
            items: rows.into_iter().map(|m| ProcessDto::from_model(m, now)).collect(),
            total,
            page: search.page,
            limit: search.limit,
            counts,
        })
    }
}
