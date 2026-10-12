use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

pub async fn begin_tx(conn: &DatabaseConnection) -> Result<DatabaseTransaction, DbErr> {
    conn.begin().await
}
