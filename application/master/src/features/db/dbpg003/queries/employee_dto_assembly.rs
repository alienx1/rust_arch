//! DBPG003 query: assembles `EmployeeDto`s (organization names, prefix, person names, account link)

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::features::db::dbpg002::models::{
    db_branch, db_branch_label, db_company, db_company_label, db_department, db_department_label,
};
use crate::features::db::dbpg002::queries::db_dbpg002_queries::fold_names;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    ENGLISH_LANGUAGE_ID, EmployeeDto, ImageOwner, THAI_LANGUAGE_ID, db_employee, db_person_name,
    db_person_name_label,
    image_url,
};
use crate::features::db::dbpg003::queries::db_dbpg003_queries::DbDbpg003Queries;
use crate::features::localization::request_language::{Language, current, pick};
use domain::entity::{db_master_lookup_label, su_user};

/// Per-language person name parts
#[derive(Default, Clone)]
struct PersonNames {
    first: String,
    middle: String,
    last: String,
    nickname: String,
}

fn join_name(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Prefix + name parts of the request's language; the other language when that name is empty
fn localized_full_name(
    prefix_th: &str,
    th: &PersonNames,
    prefix_en: &str,
    en: &PersonNames,
) -> String {
    let empty = |n: &PersonNames| n.first.is_empty() && n.last.is_empty();
    let use_english = match current() {
        Language::English => !empty(en) || empty(th),
        Language::Thai => empty(th),
    };
    if use_english {
        join_name(&[prefix_en, &en.first, &en.middle, &en.last])
    } else {
        join_name(&[prefix_th, &th.first, &th.middle, &th.last])
    }
}

impl DbDbpg003Queries {
    pub(super) async fn employee_dtos(
        &self,
        employees: Vec<db_employee::Model>,
    ) -> Result<Vec<EmployeeDto>, Dbpg003Error> {
        if employees.is_empty() {
            return Ok(Vec::new());
        }
        let ids = |f: fn(&db_employee::Model) -> Uuid| -> Vec<Uuid> {
            employees
                .iter()
                .map(f)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };
        let company_ids = ids(|e| e.company_id);
        let branch_ids = ids(|e| e.branch_id);
        let department_ids = ids(|e| e.department_id);
        let person_ids = ids(|e| e.person_name_id);
        let employee_ids = ids(|e| e.id);

        let company_codes: HashMap<Uuid, String> = db_company::Entity::find()
            .filter(db_company::Column::Id.is_in(company_ids.clone()))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.code))
            .collect();
        let company_names = fold_names(
            db_company_label::Entity::find()
                .filter(db_company_label::Column::CompanyId.is_in(company_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.company_id, l.language_id, l.label)),
        );
        let branch_codes: HashMap<Uuid, String> = db_branch::Entity::find()
            .filter(db_branch::Column::Id.is_in(branch_ids.clone()))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|b| (b.id, b.code))
            .collect();
        let branch_names = fold_names(
            db_branch_label::Entity::find()
                .filter(db_branch_label::Column::BranchId.is_in(branch_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.branch_id, l.language_id, l.label)),
        );
        let department_codes: HashMap<Uuid, String> = db_department::Entity::find()
            .filter(db_department::Column::Id.is_in(department_ids.clone()))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|d| (d.id, d.code))
            .collect();
        let department_names = fold_names(
            db_department_label::Entity::find()
                .filter(db_department_label::Column::DepartmentId.is_in(department_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.department_id, l.language_id, l.label)),
        );

        let prefix_of: HashMap<Uuid, Uuid> = db_person_name::Entity::find()
            .filter(db_person_name::Column::Id.is_in(person_ids.clone()))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|p| (p.id, p.master_lookup_id))
            .collect();
        let prefix_ids: Vec<Uuid> = prefix_of
            .values()
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let prefix_names = fold_names(
            db_master_lookup_label::Entity::find()
                .filter(db_master_lookup_label::Column::MasterLookupId.is_in(prefix_ids))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|l| (l.master_lookup_id, l.language_id, l.label)),
        );

        let mut person_names: HashMap<(Uuid, Uuid), PersonNames> = HashMap::new();
        for l in db_person_name_label::Entity::find()
            .filter(db_person_name_label::Column::PersonNameId.is_in(person_ids))
            .all(&self.db)
            .await?
        {
            person_names.insert(
                (l.person_name_id, l.language_id),
                PersonNames {
                    first: l.first_name,
                    middle: l.middle_name.unwrap_or_default(),
                    last: l.last_name,
                    nickname: l.nickname.unwrap_or_default(),
                },
            );
        }

        let with_account: HashSet<Uuid> = su_user::Entity::find()
            .filter(su_user::Column::EmployeeId.is_in(employee_ids))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|u| u.employee_id)
            .collect();

        Ok(employees
            .into_iter()
            .map(|e| {
                let th = person_names
                    .get(&(e.person_name_id, THAI_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_default();
                let en = person_names
                    .get(&(e.person_name_id, ENGLISH_LANGUAGE_ID))
                    .cloned()
                    .unwrap_or_default();
                let prefix_id = prefix_of
                    .get(&e.person_name_id)
                    .copied()
                    .unwrap_or_default();
                let (prefix_th, prefix_en) =
                    prefix_names.get(&prefix_id).cloned().unwrap_or_default();
                let full_name = localized_full_name(&prefix_th, &th, &prefix_en, &en);
                let name_of = |names: &HashMap<Uuid, (String, String)>, id: &Uuid| {
                    let (thai, english) = names.get(id).cloned().unwrap_or_default();
                    pick(&thai, &english).to_string()
                };
                EmployeeDto {
                    id: e.id,
                    organization_id: e.organization_id,
                    company_id: e.company_id,
                    company_code: company_codes
                        .get(&e.company_id)
                        .cloned()
                        .unwrap_or_default(),
                    company_name: name_of(&company_names, &e.company_id),
                    branch_id: e.branch_id,
                    branch_code: branch_codes.get(&e.branch_id).cloned().unwrap_or_default(),
                    branch_name: name_of(&branch_names, &e.branch_id),
                    department_id: e.department_id,
                    department_code: department_codes
                        .get(&e.department_id)
                        .cloned()
                        .unwrap_or_default(),
                    department_name: name_of(&department_names, &e.department_id),
                    person_name_id: e.person_name_id,
                    prefix_id,
                    prefix_name: pick(&prefix_th, &prefix_en).to_string(),
                    code: e.code,
                    full_name,
                    first_name_th: th.first,
                    middle_name_th: th.middle,
                    last_name_th: th.last,
                    nickname_th: th.nickname,
                    first_name_en: en.first,
                    middle_name_en: en.middle,
                    last_name_en: en.last,
                    nickname_en: en.nickname,
                    active: e.active,
                    has_user_account: with_account.contains(&e.id),
                    image_url: image_url(ImageOwner::Employee, e.id, e.image_key.as_deref()),
                    row_version: e.row_version,
                }
            })
            .collect())
    }
}
