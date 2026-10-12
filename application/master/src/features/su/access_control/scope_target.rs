//! Scope rules of a user: which unit a request touches and which assignments reach it.
//!
//! A user holds permission groups at scopes (`su_user_assignment`: a group at one organization /
//! company / branch / department). These functions are pure so the rules are testable without a
//! database.

use uuid::Uuid;

use domain::entity::sea_orm_active_enums::ScopeLevel;

/// The unit a request works on, as the chain of ids from its organization down. Fill in every
/// level the record is known to belong to (an employee knows all four, a branch only the first
/// three, an organization only itself).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScopeTarget {
    pub organization_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

impl ScopeTarget {
    pub fn organization(organization: Uuid) -> Self {
        Self { organization_id: Some(organization), ..Self::default() }
    }

    pub fn company(organization: Uuid, company: Uuid) -> Self {
        Self { company_id: Some(company), ..Self::organization(organization) }
    }

    pub fn branch(organization: Uuid, company: Uuid, branch: Uuid) -> Self {
        Self { branch_id: Some(branch), ..Self::company(organization, company) }
    }

    pub fn department(organization: Uuid, company: Uuid, branch: Uuid, department: Uuid) -> Self {
        Self { department_id: Some(department), ..Self::branch(organization, company, branch) }
    }

    /// `true` when the unit `unit_id` of `level` contains this target: the target carries that
    /// unit at that level (so an organization covers everything below it)
    pub fn is_inside(&self, level: &ScopeLevel, unit_id: Uuid) -> bool {
        let at_level = match level {
            ScopeLevel::Organization => self.organization_id,
            ScopeLevel::Company => self.company_id,
            ScopeLevel::Branch => self.branch_id,
            ScopeLevel::Department => self.department_id,
        };
        at_level == Some(unit_id)
    }
}

/// Whether acting on `target` is allowed, given the scopes of the user's assignments whose group
/// grants the program and action (`granting`, one entry per assignment; `None` = anywhere).
///
/// Without a target ("may the user do this anywhere?") any granting assignment is enough. With a
/// target one of them must reach it: an "anywhere" assignment, or one whose unit contains the target.
pub fn grant_applies(granting: &[Option<(ScopeLevel, Uuid)>], target: Option<&ScopeTarget>) -> bool {
    let Some(target) = target else { return !granting.is_empty() };
    granting.iter().any(|scope| match scope {
        None => true,
        Some((level, unit)) => target.is_inside(level, *unit),
    })
}
