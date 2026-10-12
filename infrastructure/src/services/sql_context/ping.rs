use sea_orm::{DatabaseConnection, DbErr};

pub async fn ping_db(conn: &DatabaseConnection) -> Result<(), DbErr> {
    conn.ping().await
}
