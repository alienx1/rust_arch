//! DBPG003 write: update an employee and its person name with optimistic lock verification

use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use uuid::Uuid;

use crate::features::optimistic_update::update_versioned;

use super::employee_input_rules::required;
use super::person_name_writer::{NameInput, write_names};
use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    EmployeeDto, UpdateEmployeeCommand, db_employee, db_person_name,
};

impl Dbpg003CommandExecutor {
    pub(super) async fn update_employee(
        &self,
        id: Uuid,
        cmd: UpdateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        let code = required(&cmd.code, "Employee code")?.to_uppercase();
        required(&cmd.first_name_th, "Thai first name")?;
        required(&cmd.last_name_th, "Thai last name")?;

        let existing = db_employee::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| Dbpg003Error::NotFound(id.to_string()))?;

        let department = self.department(cmd.department_id).await?;
        self.check_prefix(cmd.prefix_id).await?;
        if existing.code != code || existing.department_id != department.id {
            self.check_duplicate(&department, &code, Some(id)).await?;
        }

        let was_active = existing.active;
        let next_version = existing.row_version + 1;
        let person_id = existing.person_name_id;
        let txn = self.db.begin().await?;

        if let Some(person) = db_person_name::Entity::find_by_id(person_id)
            .one(&txn)
            .await?
        {
            let mut am: db_person_name::ActiveModel = person.into();
            am.master_lookup_id = Set(cmd.prefix_id);
            am.active = Set(cmd.active);
            am.change_by = Set(actor_id);
            am.row_version = Set(next_version);
            am.update(&txn).await?;
        }
        let names = NameInput {
            first_th: &cmd.first_name_th,
            middle_th: &cmd.middle_name_th,
            last_th: &cmd.last_name_th,
            nickname_th: &cmd.nickname_th,
            first_en: &cmd.first_name_en,
            middle_en: &cmd.middle_name_en,
            last_en: &cmd.last_name_en,
            nickname_en: &cmd.nickname_en,
        };
        write_names(&txn, person_id, &names, false, actor_id, next_version).await?;

        let mut am: db_employee::ActiveModel = existing.into();
        am.organization_id = Set(department.organization_id);
        am.company_id = Set(department.company_id);
        am.branch_id = Set(department.branch_id);
        am.department_id = Set(department.id);
        am.code = Set(code);
        am.active = Set(cmd.active);
        am.change_by = Set(actor_id);
        update_versioned(&txn, am, (db_employee::Column::Id, db_employee::Column::RowVersion), id, cmd.row_version).await?;
        txn.commit().await?;

        // A switched-off employee cannot keep a working account; switching the employee back on
        // brings the accounts back
        if !cmd.active {
            self.suspend_employee_accounts(id).await?;
        } else if !was_active {
            self.reactivate_employee_accounts(id).await?;
        }

        self.queries.get_employee_by_id(id).await?.ok_or_else(|| {
            Dbpg003Error::Internal("Failed to retrieve updated employee".to_string())
        })
    }
}
