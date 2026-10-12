//! SUPG004 scope lookups: code and names of every scope unit, and the unit hierarchy for paths.

use sea_orm::EntityTrait;
use std::collections::HashMap;
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::{Names, fold_names};
use crate::features::su::supg004::errors::Supg004Error;
use domain::entity::sea_orm_active_enums::ScopeLevel;
use domain::entity::{
    db_branch, db_branch_label, db_company, db_company_label, db_department, db_department_label,
    db_organization, db_organization_label,
};

pub fn scope_level_name(level: &ScopeLevel) -> &'static str {
    match level {
        ScopeLevel::Organization => "organization",
        ScopeLevel::Company => "company",
        ScopeLevel::Branch => "branch",
        ScopeLevel::Department => "department",
    }
}

/// Codes from the organization down to the scoped unit, joined with ` › `
pub(super) fn scope_path_of(
    scope_id: Option<Uuid>,
    codes: &HashMap<Uuid, (String, String, String)>,
    parents: &HashMap<Uuid, Uuid>,
) -> String {
    let mut chain = Vec::new();
    let mut cursor = scope_id;
    // The hierarchy has four levels; the bound only guards against bad data
    while let Some(id) = cursor.filter(|_| chain.len() < 4) {
        chain.push(codes.get(&id).map(|c| c.0.clone()).unwrap_or_default());
        cursor = parents.get(&id).copied();
    }
    chain.reverse();
    chain.join(" › ")
}

impl DbSupg004Queries {
    /// Code + Thai + English name of every scope unit
    pub(super) async fn scope_names(&self) -> Result<HashMap<Uuid, (String, String, String)>, Supg004Error> {
        let mut out: HashMap<Uuid, (String, String, String)> = HashMap::new();
        macro_rules! collect {
            ($entity:ident, $label:ident, $label_fk:ident) => {{
                let names: Names = fold_names(
                    $label::Entity::find()
                        .all(&self.db)
                        .await?
                        .into_iter()
                        .map(|l| (l.$label_fk, l.language_id, l.label)),
                );
                for row in $entity::Entity::find().all(&self.db).await? {
                    let (th, en) = names.get(&row.id).cloned().unwrap_or_default();
                    out.insert(row.id, (row.code.clone(), th, en));
                }
            }};
        }
        collect!(db_organization, db_organization_label, organization_id);
        collect!(db_company, db_company_label, company_id);
        collect!(db_branch, db_branch_label, branch_id);
        collect!(db_department, db_department_label, department_id);
        Ok(out)
    }

    /// Parent of every scope unit (company → organization, branch → company, department → branch)
    pub(super) async fn scope_parents(&self) -> Result<HashMap<Uuid, Uuid>, Supg004Error> {
        let mut out = HashMap::new();
        for c in db_company::Entity::find().all(&self.db).await? {
            out.insert(c.id, c.organization_id);
        }
        for b in db_branch::Entity::find().all(&self.db).await? {
            out.insert(b.id, b.company_id);
        }
        for d in db_department::Entity::find().all(&self.db).await? {
            out.insert(d.id, d.branch_id);
        }
        Ok(out)
    }
}
