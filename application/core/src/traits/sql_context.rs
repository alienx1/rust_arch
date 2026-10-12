use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr};

#[async_trait]
pub trait SqlContext: Send + Sync {
    /// Get reference to active database connection pool.
    fn connection(&self) -> &DatabaseConnection;

    /// Begin a new database transaction.
    async fn begin(&self) -> Result<DatabaseTransaction, DbErr>;

    /// Check connection health / ping database.
    async fn ping(&self) -> Result<(), DbErr>;
}
