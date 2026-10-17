//! Which part of the organization the request is looking at.
//!
//! What an account may look at is its own branch plus whatever its permission groups reach
//! (`su_user_assignment`: a group at an organization, company, branch or department), so an
//! account can work across several branches; the headquarters account sees everything. The API
//! middleware works that out per request and runs the rest of the request inside [`scope`];
//! queries over companies, branches, departments and employees call [`current`] and narrow
//! themselves. Outside a scope (background jobs, tests, accounts the middleware cannot place)
//! nothing is narrowed.

use std::future::Future;

use uuid::Uuid;

tokio::task_local! {
    static VIEW_SCOPE: ViewScope;
}

/// `None` = no limit on that level
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewScope {
    /// Only these companies
    pub companies: Option<Vec<Uuid>>,
    /// Only these branches, as a whole
    pub branches: Option<Vec<Uuid>>,
    /// Departments visible on their own, in branches that are **not** visible as a whole (a group
    /// assigned at a department reaches that department, not its whole branch)
    pub departments: Vec<Uuid>,
    /// The branches those departments are in: their rows show, their other data does not
    pub partial_branches: Vec<Uuid>,
}

impl ViewScope {
    /// Limited to one whole branch of one company
    pub fn one_branch(company: Uuid, branch: Uuid) -> Self {
        Self { companies: Some(vec![company]), branches: Some(vec![branch]), ..Self::default() }
    }

    /// Limited to one company
    pub fn one_company(company: Uuid) -> Self {
        Self { companies: Some(vec![company]), ..Self::default() }
    }

    /// Whether anything is limited (an overview scope limits nothing)
    pub fn is_limited(&self) -> bool {
        self.companies.is_some() || self.branches.is_some()
    }

    /// Branch rows the scope shows: the whole ones and the partial ones (`None` = every branch)
    pub fn branch_rows(&self) -> Option<Vec<Uuid>> {
        self.branches.as_ref().map(|b| b.iter().chain(&self.partial_branches).copied().collect())
    }

    /// The only company, when the scope is down to one
    pub fn single_company(&self) -> Option<Uuid> {
        match self.companies.as_deref() {
            Some([one]) => Some(*one),
            _ => None,
        }
    }

    /// The only branch (shown as a whole), when the scope is down to one and nothing else shows
    pub fn single_branch(&self) -> Option<Uuid> {
        match (self.branches.as_deref(), self.partial_branches.is_empty()) {
            (Some([one]), true) => Some(*one),
            _ => None,
        }
    }
}

/// Runs `future` with `view` as what the request may look at
pub async fn scope<F: Future>(view: ViewScope, future: F) -> F::Output {
    VIEW_SCOPE.scope(view, future).await
}

/// What the request may look at (`None` outside a request: unlimited)
pub fn current() -> Option<ViewScope> {
    VIEW_SCOPE.try_with(|v| v.clone()).ok()
}

/// The companies the request is limited to, if any
pub fn companies() -> Option<Vec<Uuid>> {
    current().and_then(|v| v.companies)
}

/// Text that tells scopes apart, for cache keys (`-` when unlimited)
pub fn signature() -> String {
    let list = |ids: &[Uuid]| ids.iter().map(|i| i.simple().to_string()).collect::<Vec<_>>().join(",");
    match current() {
        None => "-".to_string(),
        Some(v) => format!(
            "{}.{}.{}.{}",
            v.companies.as_deref().map(list).unwrap_or_else(|| "*".to_string()),
            v.branches.as_deref().map(list).unwrap_or_else(|| "*".to_string()),
            list(&v.partial_branches),
            list(&v.departments)
        ),
    }
}
