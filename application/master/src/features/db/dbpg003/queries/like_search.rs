//! DBPG003 LIKE search helpers: literal escaping and case-insensitive contains

use sea_orm::ColumnTrait;
use sea_orm::sea_query::{Expr, ExprTrait, Func};

/// Escapes `\`, `%` and `_` so user input is matched literally by LIKE.
pub(crate) fn escape_like(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Case-insensitive `column LIKE %term%` expression.
pub(crate) fn contains_ci<C: ColumnTrait>(column: C, like: &str) -> sea_orm::sea_query::SimpleExpr {
    Expr::expr(Func::lower(Expr::col(column))).like(like.to_string())
}
