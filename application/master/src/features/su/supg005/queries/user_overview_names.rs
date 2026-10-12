//! SUPG005 overview names: employee, department, company and role names in the request language.

use domain::entity::{db_company_label, db_department_label, db_employee, db_person_name_label};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::features::db::dbpg000::models::{ENGLISH_LANGUAGE_ID, THAI_LANGUAGE_ID};
use crate::features::localization::request_language::pick;
use crate::features::su::supg005::errors::Supg005Error;
use crate::features::su::supg005::queries::DbSupg005Queries;

/// Role name shown for a permission group code
pub(super) fn role_display_name(role_code: &str) -> String {
    pick(&format!("บทบาท {role_code}"), &format!("Role {role_code}")).to_string()
}

fn label_of<'a>(rows: impl Iterator<Item = (Uuid, &'a str)>) -> String {
    let (mut th, mut en) = (String::new(), String::new());
    for (language, label) in rows {
        if language == THAI_LANGUAGE_ID {
            th = label.to_string();
        } else if language == ENGLISH_LANGUAGE_ID {
            en = label.to_string();
        }
    }
    pick(&th, &en).to_string()
}

impl DbSupg005Queries {
    /// Names of `employee` in the request language; `None` fields stay at the caller's defaults
    pub(super) async fn employee_overview_names(
        &self,
        employee: &db_employee::Model,
    ) -> Result<(Option<String>, Option<String>, Option<String>), Supg005Error> {
        let person = db_person_name_label::Entity::find()
            .filter(db_person_name_label::Column::PersonNameId.eq(employee.person_name_id))
            .all(&self.db)
            .await?;
        let full = |language: Uuid| {
            person
                .iter()
                .find(|l| l.language_id == language)
                .map(|l| format!("{} {}", l.first_name, l.last_name).trim().to_string())
                .unwrap_or_default()
        };
        let full_name = pick(&full(THAI_LANGUAGE_ID), &full(ENGLISH_LANGUAGE_ID)).to_string();

        let departments = db_department_label::Entity::find()
            .filter(db_department_label::Column::DepartmentId.eq(employee.department_id))
            .all(&self.db)
            .await?;
        let department_name = label_of(departments.iter().map(|l| (l.language_id, l.label.as_str())));

        let companies = db_company_label::Entity::find()
            .filter(db_company_label::Column::CompanyId.eq(employee.company_id))
            .all(&self.db)
            .await?;
        let company_name = label_of(companies.iter().map(|l| (l.language_id, l.label.as_str())));

        let some = |s: String| (!s.is_empty()).then_some(s);
        Ok((some(full_name), some(department_name), some(company_name)))
    }
}
