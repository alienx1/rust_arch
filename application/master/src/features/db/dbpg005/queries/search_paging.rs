//! Paging and search-term helpers shared by the geography list queries

use crate::features::db::dbpg003::queries::db_dbpg003_queries::escape_like;
use crate::features::db::dbpg005::models::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

pub(super) fn page_and_limit(page: Option<u64>, limit: Option<u64>) -> (u64, u64) {
    (Ord::max(page.unwrap_or(1), 1), Ord::clamp(limit.unwrap_or(DEFAULT_PAGE_SIZE), 1, MAX_PAGE_SIZE))
}

/// `LIKE %term%` pattern for a user search term, `None` when blank
pub(super) fn like_of(search: &Option<String>) -> Option<String> {
    let term = search.as_deref().unwrap_or("").trim().to_lowercase();
    (!term.is_empty()).then(|| format!("%{}%", escape_like(&term)))
}
