//! SUPG004 scope-options query: organizations, companies, branches and departments for the
//! assignment picker.

use sea_orm::EntityTrait;
use uuid::Uuid;

use super::db_supg004_queries::DbSupg004Queries;
use crate::features::db::dbpg002::queries::db_dbpg002_queries::{Names, fold_names};
use crate::features::su::supg004::errors::Supg004Error;
use crate::features::su::supg004::models::{ScopeOptionDto, ScopeOptionsDto};
use domain::entity::{
    db_branch, db_branch_label, db_company, db_company_label, db_department, db_department_label,
    db_organization, db_organization_label,
};

fn scope_options_of<M>(
    rows: Vec<M>,
    id_code: impl Fn(&M) -> (Uuid, String, Option<Uuid>),
    names: &Names,
) -> Vec<ScopeOptionDto> {
    let mut out: Vec<ScopeOptionDto> = rows
        .iter()
        .map(|m| {
            let (id, code, parent_id) = id_code(m);
            let (name_th, name_en) = names.get(&id).cloned().unwrap_or_default();
            ScopeOptionDto { id, code, name_th, name_en, parent_id }
        })
        .collect();
    out.sort_by(|a, b| a.code.cmp(&b.code));
    out
}

impl DbSupg004Queries {
    pub(super) async fn scope_options(&self) -> Result<ScopeOptionsDto, Supg004Error> {
        macro_rules! options {
            ($entity:ident, $label:ident, $label_fk:ident, $parent:expr) => {{
                let names = fold_names(
                    $label::Entity::find()
                        .all(&self.db)
                        .await?
                        .into_iter()
                        .map(|l| (l.$label_fk, l.language_id, l.label)),
                );
                scope_options_of($entity::Entity::find().all(&self.db).await?, |m| (m.id, m.code.clone(), $parent(m)), &names)
            }};
        }
        Ok(ScopeOptionsDto {
            organizations: options!(db_organization, db_organization_label, organization_id, |_: &db_organization::Model| None),
            companies: options!(db_company, db_company_label, company_id, |m: &db_company::Model| Some(m.organization_id)),
            branches: options!(db_branch, db_branch_label, branch_id, |m: &db_branch::Model| Some(m.company_id)),
            departments: options!(db_department, db_department_label, department_id, |m: &db_department::Model| Some(m.branch_id)),
        })
    }
}
