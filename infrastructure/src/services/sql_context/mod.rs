//! pgEdge (PostgreSQL 17 / Spock multi-master) Database Context implementation.

pub mod audit_actor;
pub mod begin;
pub mod connect;
pub mod ping;

use application_core::traits::SqlContext;
use async_trait::async_trait;
use sea_orm::{ConnectOptions, DatabaseConnection, DatabaseTransaction, DbErr};

#[derive(Clone)]
pub struct PgEdgeSqlContext {
    conn: DatabaseConnection,
}

impl PgEdgeSqlContext {
    /// Connect to pgEdge database with optimal production connection pool settings.
    pub async fn connect(database_url: &str) -> Result<Self, DbErr> {
        let conn = connect::connect_pgedge(database_url).await?;
        Ok(Self { conn })
    }

    /// Connect with custom `ConnectOptions`.
    pub async fn with_options(opt: ConnectOptions) -> Result<Self, DbErr> {
        let conn = connect::connect_with_options(opt).await?;
        Ok(Self { conn })
    }

    /// Construct from an existing DatabaseConnection.
    pub fn from_connection(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SqlContext for PgEdgeSqlContext {
    fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        begin::begin_tx(&self.conn).await
    }

    async fn ping(&self) -> Result<(), DbErr> {
        ping::ping_db(&self.conn).await
    }
}
