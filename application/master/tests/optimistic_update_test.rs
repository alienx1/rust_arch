//! `update_versioned`: one UPDATE that carries the loaded `row_version`; a row at another version
//! (or gone) matches nothing and fails with `RecordNotUpdated`, which the features map to
//! `VersionConflict`. Located at `application/master/tests/optimistic_update_test.rs`.

use application_master::features::optimistic_update::update_versioned;
use domain::entity::db_branch;
use sea_orm::{ActiveValue::Set, ActiveValue::Unchanged, DatabaseBackend, DbErr, MockDatabase, MockExecResult};
use uuid::Uuid;

fn branch(id: Uuid) -> db_branch::ActiveModel {
    db_branch::ActiveModel {
        id: Unchanged(id),
        code: Set("B001".to_string()),
        // A version set by the caller is ignored: the helper moves it up itself
        row_version: Set(99),
        ..Default::default()
    }
}

fn columns() -> (db_branch::Column, db_branch::Column) {
    (db_branch::Column::Id, db_branch::Column::RowVersion)
}

#[tokio::test]
async fn test_the_update_is_conditional_on_the_loaded_version_and_bumps_it() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult { last_insert_id: 0, rows_affected: 1 }])
        .into_connection();
    let id = Uuid::now_v7();
    update_versioned(&db, branch(id), columns(), id, 5).await.unwrap();

    let sql = format!("{:?}", db.into_transaction_log()).replace("\\\"", "\"");
    assert!(sql.contains(r#""row_version" = "row_version" + $2"#) && sql.contains("Int(Some(1))"), "bumps in SQL: {sql}");
    assert!(
        sql.contains(r#"AND "db_branch"."row_version" = $4"#) && sql.contains("BigInt(Some(5))"),
        "guarded by the loaded version: {sql}"
    );
    assert!(!sql.contains("99"), "a version set by the caller is ignored: {sql}");
}

#[tokio::test]
async fn test_a_row_at_another_version_is_a_record_not_updated_error() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult { last_insert_id: 0, rows_affected: 0 }])
        .into_connection();
    let id = Uuid::now_v7();
    let result = update_versioned(&db, branch(id), columns(), id, 5).await;
    assert!(matches!(result, Err(DbErr::RecordNotUpdated)));
}
