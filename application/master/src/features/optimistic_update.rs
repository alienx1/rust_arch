//! Optimistic locking in one place: a save that carries the `row_version` the user loaded.
//!
//! `update_versioned` runs `UPDATE … SET <changed columns>, row_version = row_version + 1 WHERE id = ?
//! AND row_version = <loaded>` as one statement, so a row that somebody else saved (or removed)
//! since it was loaded matches nothing and the save fails with [`DbErr::RecordNotUpdated`], which
//! every feature error maps to its `VersionConflict`. Callers neither compare versions nor add one:
//! they set the columns that change and hand over the version the client sent.

use sea_orm::sea_query::{Expr, ExprTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

/// Updates the `Set` columns of `model` on the row `id` if it is still at version `expected`, and
/// moves its `row_version` up by one. A `row_version` set on `model` is ignored.
pub async fn update_versioned<E, A, C>(
    conn: &C,
    mut model: A,
    (id_column, version_column): (E::Column, E::Column),
    id: Uuid,
    expected: i64,
) -> Result<(), DbErr>
where
    E: EntityTrait,
    E::Column: Copy,
    A: ActiveModelTrait<Entity = E>,
    C: ConnectionTrait,
{
    model.not_set(version_column);
    let result = E::update_many()
        .set(model)
        .col_expr(version_column, Expr::col(version_column).add(1))
        .filter(id_column.eq(id))
        .filter(version_column.eq(expected))
        .exec(conn)
        .await?;
    if result.rows_affected == 1 { Ok(()) } else { Err(DbErr::RecordNotUpdated) }
}
