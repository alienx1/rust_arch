//! Narrows queries over companies, branches, departments and employees to what the request may
//! look at (`application_core::view_scope`): the whole organization for headquarters; for everybody
//! else their own branch plus the branches (or just departments) their permission groups reach.
//! Outside a request nothing is narrowed.

use application_core::view_scope::{self, ViewScope};
use sea_orm::{ColumnTrait, Condition, QueryFilter};
use uuid::Uuid;

/// Keeps only rows of the companies the request is limited to; `column` is the company column of
/// the table (`db_company.id` for the companies themselves)
pub fn by_company<S: QueryFilter>(select: S, column: impl ColumnTrait) -> S {
    match view_scope::companies() {
        Some(companies) => select.filter(column.is_in(companies)),
        None => select,
    }
}

/// For the branches themselves: keeps the branches the request shows (whole or partly);
/// `column` is `db_branch.id`
pub fn by_branch_rows<S: QueryFilter>(select: S, column: impl ColumnTrait) -> S {
    match view_scope::current().and_then(|v| v.branch_rows()) {
        Some(branches) => select.filter(column.is_in(branches)),
        None => select,
    }
}

/// The condition for rows that belong to a branch and a department (departments, employees): in a
/// branch shown as a whole, or in a department the request was granted on its own. `None` when
/// nothing is limited.
pub fn branch_data_condition(branch_column: impl ColumnTrait, department_column: impl ColumnTrait) -> Option<Condition> {
    let ViewScope { branches, departments, .. } = view_scope::current()?;
    let branches = branches?;
    let mut any = Condition::any().add(branch_column.is_in(branches));
    if !departments.is_empty() {
        any = any.add(department_column.is_in(departments));
    }
    Some(any)
}

/// [`branch_data_condition`] applied to a select
pub fn by_branch_data<S: QueryFilter>(select: S, branch_column: impl ColumnTrait, department_column: impl ColumnTrait) -> S {
    match branch_data_condition(branch_column, department_column) {
        Some(cond) => select.filter(cond),
        None => select,
    }
}

/// Whether a row with these ids is inside what the request may look at. A row without a branch or
/// department (`None`) cannot be excluded on that level.
pub fn in_scope(company_id: Option<Uuid>, branch_id: Option<Uuid>, department_id: Option<Uuid>) -> bool {
    let Some(scope) = view_scope::current() else { return true };
    let company_ok = match (&scope.companies, company_id) {
        (Some(allowed), Some(id)) => allowed.contains(&id),
        _ => true,
    };
    let branch_ok = match (&scope.branches, branch_id) {
        (Some(allowed), Some(id)) => {
            allowed.contains(&id) || department_id.is_some_and(|d| scope.departments.contains(&d))
        }
        _ => true,
    };
    company_ok && branch_ok
}

/// Whether a branch row is shown to the request (whole or partly)
pub fn branch_in_scope(company_id: Uuid, branch_id: Uuid) -> bool {
    let Some(scope) = view_scope::current() else { return true };
    let company_ok = scope.companies.as_ref().is_none_or(|c| c.contains(&company_id));
    let branch_ok = scope.branch_rows().is_none_or(|b| b.contains(&branch_id));
    company_ok && branch_ok
}
