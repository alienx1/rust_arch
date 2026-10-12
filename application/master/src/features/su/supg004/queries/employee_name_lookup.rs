//! SUPG004 employee name lookups: full names in the request language for display, English-first names for the
//! identity provider.

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashMap;
use uuid::Uuid;

use super::EmployeeNames;
use super::db_supg004_queries::DbSupg004Queries;
use crate::features::db::dbpg000::models::{ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID};
use crate::features::localization::request_language::pick as pick_name;
use crate::features::su::supg004::errors::Supg004Error;
use domain::entity::{db_employee, db_person_name_label};

fn join_name(parts: &[&str]) -> String {
    parts.iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ")
}

impl DbSupg004Queries {
    /// Full name of employees by employee id, in the request language (other language when empty)
    pub(super) async fn full_names(&self, employees: &[db_employee::Model]) -> Result<HashMap<Uuid, String>, Supg004Error> {
        let person_ids: Vec<Uuid> = employees.iter().map(|e| e.person_name_id).collect();
        // (Thai, English) full name per person
        let mut by_person: HashMap<Uuid, (String, String)> = HashMap::new();
        if !person_ids.is_empty() {
            for l in db_person_name_label::Entity::find()
                .filter(db_person_name_label::Column::PersonNameId.is_in(person_ids))
                .all(&self.db)
                .await?
            {
                let full = join_name(&[&l.first_name, l.middle_name.as_deref().unwrap_or(""), &l.last_name]);
                let entry = by_person.entry(l.person_name_id).or_default();
                if l.language_id == THAI_LANGUAGE_ID {
                    entry.0 = full;
                } else if l.language_id == ENGLISH_LANGUAGE_ID {
                    entry.1 = full;
                }
            }
        }
        Ok(employees
            .iter()
            .map(|e| {
                let (th, en) = by_person.get(&e.person_name_id).cloned().unwrap_or_default();
                (e.id, pick_name(&th, &en).to_string())
            })
            .collect())
    }

    pub(super) async fn employee_names(&self, employee_id: Uuid) -> Result<Option<EmployeeNames>, Supg004Error> {
        let Some(employee) = db_employee::Entity::find_by_id(employee_id).one(&self.db).await? else {
            return Ok(None);
        };
        let labels = db_person_name_label::Entity::find()
            .filter(db_person_name_label::Column::PersonNameId.eq(employee.person_name_id))
            .all(&self.db)
            .await?;
        let pick = |language: Uuid| labels.iter().find(|l| l.language_id == language);
        Ok(pick(ENGLISH_LANGUAGE_ID)
            .or_else(|| pick(THAI_LANGUAGE_ID))
            .map(|l| EmployeeNames { first_name: l.first_name.clone(), last_name: l.last_name.clone() }))
    }
}
