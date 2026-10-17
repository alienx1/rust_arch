//! Where an account works and what it may look at.
//!
//! The headquarters (branch `HQ`) and any account holding a permission group at the whole
//! organization see the organization. Everybody else sees their own branch plus what their
//! permission groups reach (`su_user_assignment`): a group at a company reaches its branches, at a
//! branch that branch, at a department that department only (its branch shows as a row, its other
//! data stays hidden). So one account can work across several branches.

use application_core::view_scope::ViewScope;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{
    db_branch, db_branch_label, db_company, db_company_label, db_department, db_employee, db_organization,
    db_organization_label, su_permission_group, su_user, su_user_assignment,
};

use super::{AccessError, DbAccessControl, OwnBranch, OwnEmployee, OwnUnit};
use crate::features::db::dbpg000::models::{ENGLISH_LANGUAGE_ID, ImageOwner, THAI_LANGUAGE_ID, image_url};

/// Code of the branch that counts as the headquarters
pub const HQ_BRANCH_CODE: &str = "HQ";

/// A branch an account can reach
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReachedBranch {
    pub branch_id: Uuid,
    pub company_id: Uuid,
    /// The whole branch is visible (its own branch, or a group at the branch / its company);
    /// `false` = only the departments listed in [`Viewer::departments`]
    pub whole: bool,
}

/// What one permission group the account holds reaches
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupReach {
    pub group_id: Uuid,
    /// The group is assigned at the whole organization
    pub unlimited: bool,
    pub branches: Vec<ReachedBranch>,
    /// Departments reached on their own, with their branch
    pub departments: Vec<(Uuid, Uuid)>,
}

/// Where the account works and what it reaches
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewer {
    pub organization_id: Uuid,
    pub company_id: Uuid,
    pub branch_id: Uuid,
    /// The branch is the headquarters
    pub is_hq: bool,
    /// Sees the whole organization: headquarters, or a group assigned at the organization
    pub unlimited: bool,
    /// Branches reached by everything the account holds (own branch included); unused when
    /// `unlimited`
    pub branches: Vec<ReachedBranch>,
    /// Departments reached on their own, with their branch
    pub departments: Vec<(Uuid, Uuid)>,
    /// The same reach per permission group, so a screen can be limited to where the account holds
    /// the permission for that screen
    pub reaches: Vec<GroupReach>,
}

impl Viewer {
    /// A viewer who is limited to its own branch (no other reach)
    pub fn own_branch(organization_id: Uuid, company_id: Uuid, branch_id: Uuid) -> Self {
        Self {
            organization_id,
            company_id,
            branch_id,
            is_hq: false,
            unlimited: false,
            branches: vec![ReachedBranch { branch_id, company_id, whole: true }],
            departments: Vec::new(),
            reaches: Vec::new(),
        }
    }
}

/// What `viewer` may look at, given what it picked in the switcher.
///
/// `groups` are the permission groups that grant the screen being served (`None` = no screen in
/// particular: everything the account holds counts, the headquarters sees the organization, and a
/// pick outside its reach is ignored). With groups, only where those groups reach counts, so a
/// screen shows data only from the branches where the account holds its permission (even for the
/// headquarters), and a pick outside that reach is refused (`None`).
///
/// Unlimited viewers (headquarters, or a group at the whole organization): the whole organization,
/// one company when `company` is given, or one branch when `branch` is given (a branch and its
/// company, both already checked to belong to the organization; the branch wins over a company
/// that disagrees).
///
/// Everybody else: the branches they reach, narrowed by the pick when the pick is one of them. A
/// branch reached only through departments shows those departments only.
pub fn view_scope_for(
    viewer: &Viewer,
    groups: Option<&[Uuid]>,
    company: Option<Uuid>,
    branch: Option<(Uuid, Uuid)>,
) -> Option<ViewScope> {
    let (unlimited, reached, departments) = if let Some(groups) = groups {
        let chosen: Vec<&GroupReach> = viewer.reaches.iter().filter(|r| groups.contains(&r.group_id)).collect();
        let mut merged: Vec<ReachedBranch> = Vec::new();
        for r in chosen.iter().flat_map(|r| &r.branches) {
            match merged.iter_mut().find(|m| m.branch_id == r.branch_id) {
                Some(m) => m.whole |= r.whole,
                None => merged.push(*r),
            }
        }
        let departments = chosen.iter().flat_map(|r| r.departments.iter().copied()).collect();
        (chosen.iter().any(|r| r.unlimited), merged, departments)
    } else {
        (viewer.unlimited, viewer.branches.clone(), viewer.departments.clone())
    };
    let strict = groups.is_some();

    if unlimited {
        return Some(match branch {
            Some((branch_id, company_id)) => ViewScope::one_branch(company_id, branch_id),
            None => ViewScope { companies: company.map(|c| vec![c]), ..ViewScope::default() },
        });
    }

    let mut reached = reached;
    match branch {
        Some((branch_id, _)) if reached.iter().any(|r| r.branch_id == branch_id) => {
            reached.retain(|r| r.branch_id == branch_id)
        }
        Some(_) if strict => return None,
        _ => match company {
            Some(company_id) if reached.iter().any(|r| r.company_id == company_id) => {
                reached.retain(|r| r.company_id == company_id)
            }
            Some(_) if strict && branch.is_none() => return None,
            _ => {}
        },
    }

    let companies: BTreeSet<Uuid> = reached.iter().map(|r| r.company_id).collect();
    let whole: Vec<Uuid> = reached.iter().filter(|r| r.whole).map(|r| r.branch_id).collect();
    let partial: Vec<Uuid> = reached.iter().filter(|r| !r.whole).map(|r| r.branch_id).collect();
    let departments = departments.into_iter().filter(|(_, b)| partial.contains(b)).map(|(d, _)| d).collect();
    Some(ViewScope {
        companies: Some(companies.into_iter().collect()),
        branches: Some(whole),
        departments,
        partial_branches: partial,
    })
}

/// Thai and English text out of the labels of one unit
fn names<'a>(labels: impl Iterator<Item = (Uuid, &'a str)>) -> (String, String) {
    let (mut th, mut en) = (String::new(), String::new());
    for (language, label) in labels {
        if language == THAI_LANGUAGE_ID {
            th = label.to_string();
        } else if language == ENGLISH_LANGUAGE_ID {
            en = label.to_string();
        }
    }
    (th, en)
}

impl DbAccessControl {
    /// The employee of an active account, with the branch it works in
    async fn employee_of(&self, username: &str) -> Result<Option<(su_user::Model, db_employee::Model, bool)>, AccessError> {
        let Some(user) = su_user::Entity::find().filter(su_user::Column::Username.eq(username)).one(&self.db).await?
        else {
            return Ok(None);
        };
        if !user.active {
            return Ok(None);
        }
        let Some(employee) = db_employee::Entity::find_by_id(user.employee_id).one(&self.db).await? else {
            return Ok(None);
        };
        let is_hq = db_branch::Entity::find_by_id(employee.branch_id)
            .one(&self.db)
            .await?
            .is_some_and(|b| b.code.eq_ignore_ascii_case(HQ_BRANCH_CODE));
        Ok(Some((user, employee, is_hq)))
    }

    /// What the account reaches, in all and per permission group:
    /// `(unlimited, branches, departments, per group)`
    async fn reach_of(
        &self,
        user: &su_user::Model,
        e: &db_employee::Model,
        is_hq: bool,
    ) -> Result<(bool, Vec<ReachedBranch>, Vec<(Uuid, Uuid)>, Vec<GroupReach>), AccessError> {
        let own = ReachedBranch { branch_id: e.branch_id, company_id: e.company_id, whole: true };
        let assignments =
            su_user_assignment::Entity::find().filter(su_user_assignment::Column::UserId.eq(user.id)).all(&self.db).await?;
        if assignments.is_empty() {
            // No assignment rows: the primary group applies, here (the account's own branch)
            // (the headquarters, which has nothing assigned, sees the whole organization)
            let reach = GroupReach { group_id: user.permission_group_id, unlimited: is_hq, branches: vec![own], departments: vec![] };
            return Ok((is_hq, vec![own], Vec::new(), vec![reach]));
        }
        // Only groups that are switched on reach anything
        let active_groups: std::collections::HashSet<Uuid> = su_permission_group::Entity::find()
            .filter(su_permission_group::Column::Id.is_in(assignments.iter().map(|a| a.permission_group_id)))
            .filter(su_permission_group::Column::Active.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|g| g.id)
            .collect();

        let mut per_group: HashMap<Uuid, GroupReach> = HashMap::new();
        for a in assignments.into_iter().filter(|a| active_groups.contains(&a.permission_group_id)) {
            let reach = per_group.entry(a.permission_group_id).or_insert_with(|| GroupReach {
                group_id: a.permission_group_id,
                unlimited: false,
                branches: Vec::new(),
                departments: Vec::new(),
            });
            let mut add = |branch_id: Uuid, company_id: Uuid, whole: bool| match reach.branches.iter_mut().find(|r| r.branch_id == branch_id) {
                Some(r) => r.whole |= whole,
                None => reach.branches.push(ReachedBranch { branch_id, company_id, whole }),
            };
            match (a.scope_level, a.organization_id, a.company_id, a.branch_id, a.department_id) {
                (ScopeLevel::Organization, Some(org), ..) => {
                    if org == e.organization_id {
                        reach.unlimited = true;
                    }
                }
                (ScopeLevel::Company, _, Some(company), ..) => {
                    for b in db_branch::Entity::find().filter(db_branch::Column::CompanyId.eq(company)).all(&self.db).await? {
                        add(b.id, b.company_id, true);
                    }
                }
                (ScopeLevel::Branch, _, _, Some(branch), _) => {
                    if let Some(b) = db_branch::Entity::find_by_id(branch).one(&self.db).await? {
                        add(b.id, b.company_id, true);
                    }
                }
                (ScopeLevel::Department, _, _, _, Some(department)) => {
                    if let Some(d) = db_department::Entity::find_by_id(department).one(&self.db).await? {
                        add(d.branch_id, d.company_id, false);
                        reach.departments.push((d.id, d.branch_id));
                    }
                }
                _ => {}
            }
        }
        let mut reaches: Vec<GroupReach> = per_group.into_values().collect();
        for r in &mut reaches {
            // A department inside a branch the group reaches as a whole adds nothing on its own
            let whole: Vec<Uuid> = r.branches.iter().filter(|b| b.whole).map(|b| b.branch_id).collect();
            r.departments.retain(|(_, branch)| !whole.contains(branch));
            r.branches.sort_by_key(|b| b.branch_id);
        }
        reaches.sort_by_key(|r| r.group_id);

        // In all: the own branch plus everything a group reaches
        let mut merged: HashMap<Uuid, ReachedBranch> = HashMap::new();
        for b in std::iter::once(&own).chain(reaches.iter().flat_map(|r| r.branches.iter())) {
            merged.entry(b.branch_id).and_modify(|m| m.whole |= b.whole).or_insert(*b);
        }
        let mut branches: Vec<ReachedBranch> = merged.into_values().collect();
        branches.sort_by_key(|r| r.branch_id);
        let departments = reaches
            .iter()
            .flat_map(|r| r.departments.iter().copied())
            .filter(|(_, branch)| branches.iter().any(|b| b.branch_id == *branch && !b.whole))
            .collect();
        // The headquarters looks at the whole organization; what each of its screens may show
        // still follows where its groups are assigned
        Ok((is_hq || reaches.iter().any(|r| r.unlimited), branches, departments, reaches))
    }

    pub(super) async fn load_viewer(&self, username: &str) -> Result<Option<Viewer>, AccessError> {
        let Some((user, e, is_hq)) = self.employee_of(username).await? else {
            return Ok(None);
        };
        let (unlimited, branches, departments, reaches) = self.reach_of(&user, &e, is_hq).await?;
        Ok(Some(Viewer {
            organization_id: e.organization_id,
            company_id: e.company_id,
            branch_id: e.branch_id,
            is_hq,
            unlimited,
            branches,
            departments,
            reaches,
        }))
    }

    pub(super) async fn load_own_employee(&self, username: &str) -> Result<Option<OwnEmployee>, AccessError> {
        let Some((user, e, is_hq)) = self.employee_of(username).await? else {
            return Ok(None);
        };
        let (unlimited, reach, _, _) = self.reach_of(&user, &e, is_hq).await?;
        let reached_companies: Vec<Uuid> = reach.iter().map(|r| r.company_id).collect();
        let reached_branches: Vec<Uuid> = reach.iter().map(|r| r.branch_id).collect();

        let organization = match db_organization::Entity::find_by_id(e.organization_id).one(&self.db).await? {
            Some(o) => {
                let labels = db_organization_label::Entity::find()
                    .filter(db_organization_label::Column::OrganizationId.eq(o.id))
                    .all(&self.db)
                    .await?;
                let (name_th, name_en) = names(labels.iter().map(|l| (l.language_id, l.label.as_str())));
                Some(OwnUnit {
                    id: o.id,
                    code: o.code.clone(),
                    name_th,
                    name_en,
                    image_url: image_url(ImageOwner::Organization, o.id, o.image_key.as_deref()),
                })
            }
            None => None,
        };

        // The companies the account may pick: the whole organization for headquarters
        let mut select = db_company::Entity::find().order_by_asc(db_company::Column::Code);
        select = if unlimited {
            select.filter(db_company::Column::OrganizationId.eq(e.organization_id))
        } else {
            select.filter(db_company::Column::Id.is_in(reached_companies))
        };
        let companies = select.all(&self.db).await?;
        let mut labels: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
        if !companies.is_empty() {
            for l in db_company_label::Entity::find()
                .filter(db_company_label::Column::CompanyId.is_in(companies.iter().map(|c| c.id)))
                .all(&self.db)
                .await?
            {
                labels.entry(l.company_id).or_default().push((l.language_id, l.label));
            }
        }
        let choices: Vec<OwnUnit> = companies
            .into_iter()
            .map(|c| {
                let (name_th, name_en) =
                    names(labels.get(&c.id).into_iter().flatten().map(|(lang, label)| (*lang, label.as_str())));
                OwnUnit { id: c.id, code: c.code.clone(), name_th, name_en, image_url: image_url(ImageOwner::Company, c.id, c.image_key.as_deref()) }
            })
            .collect();
        let company = choices.iter().find(|c| c.id == e.company_id).cloned();

        // The branches the account may pick
        let mut select = db_branch::Entity::find().order_by_asc(db_branch::Column::Code);
        select = if unlimited {
            select.filter(db_branch::Column::OrganizationId.eq(e.organization_id))
        } else {
            select.filter(db_branch::Column::Id.is_in(reached_branches))
        };
        let branch_rows = select.all(&self.db).await?;
        let mut branch_labels: HashMap<Uuid, Vec<(Uuid, String)>> = HashMap::new();
        if !branch_rows.is_empty() {
            for l in db_branch_label::Entity::find()
                .filter(db_branch_label::Column::BranchId.is_in(branch_rows.iter().map(|b| b.id)))
                .all(&self.db)
                .await?
            {
                branch_labels.entry(l.branch_id).or_default().push((l.language_id, l.label));
            }
        }
        let branches: Vec<OwnBranch> = branch_rows
            .into_iter()
            .map(|b| {
                let (name_th, name_en) =
                    names(branch_labels.get(&b.id).into_iter().flatten().map(|(lang, label)| (*lang, label.as_str())));
                OwnBranch { id: b.id, company_id: b.company_id, code: b.code, name_th, name_en }
            })
            .collect();

        Ok(Some(OwnEmployee {
            id: e.id,
            image_url: image_url(ImageOwner::Employee, e.id, e.image_key.as_deref()),
            organization,
            company,
            is_hq,
            unlimited,
            choices,
            branches,
        }))
    }
}
