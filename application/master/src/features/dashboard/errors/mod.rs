use thiserror::Error;

/// Feature-specific error enum for Dashboard operations.
#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Database error while fetching dashboard statistics: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal dashboard error: {0}")]
    Internal(String),
}
