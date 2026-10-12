//! `PermissionGate`: shows its children only when the account holds `action` on `program`.

use leptos::prelude::*;

use super::access_context::{Action, can};

/// ```ignore
/// <PermissionGate program="DBPG004" action=Action::Write>
///     <Button>"+ เพิ่ม"</Button>
/// </PermissionGate>
/// ```
#[component]
pub fn PermissionGate(
    program: &'static str,
    action: Action,
    children: ChildrenFn,
) -> impl IntoView {
    let allowed = can(program, action);
    view! { <Show when=move || allowed.get()>{children()}</Show> }
}
