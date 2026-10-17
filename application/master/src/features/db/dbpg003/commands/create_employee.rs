//! DBPG003 write: create an employee with its person name (one transaction)

use sea_orm::{ActiveModelTrait, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use super::employee_input_rules::required;
use super::person_name_writer::{NameInput, write_names};
use crate::features::db::dbpg003::commands::dbpg003_executor::Dbpg003CommandExecutor;
use crate::features::db::dbpg003::errors::Dbpg003Error;
use crate::features::db::dbpg003::models::{
    CreateEmployeeCommand, EmployeeDto, db_employee, db_person_name,
};

impl Dbpg003CommandExecutor {
    pub(super) async fn create_employee(
        &self,
        cmd: CreateEmployeeCommand,
        actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        let code = required(&cmd.code, "Employee code")?.to_uppercase();
        required(&cmd.first_name_th, "Thai first name")?;
        required(&cmd.last_name_th, "Thai last name")?;

        let department = self.department(cmd.department_id).await?;
        self.check_prefix(cmd.prefix_id).await?;
        self.check_duplicate(&department, &code, None).await?;

        let id = Uuid::now_v7();
        let person_id = Uuid::now_v7();
        info!("Creating new employee: id={id}, code={code}");

        let txn = self.db.begin().await?;
        db_person_name::ActiveModel {
            id: Set(person_id),
            master_lookup_id: Set(cmd.prefix_id),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
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
        write_names(&txn, person_id, &names, true, actor_id, 1).await?;
        db_employee::ActiveModel {
            id: Set(id),
            organization_id: Set(department.organization_id),
            company_id: Set(department.company_id),
            branch_id: Set(department.branch_id),
            department_id: Set(department.id),
            person_name_id: Set(person_id),
            address_group_id: Set(None),
            contact_group_id: Set(None),
            code: Set(code),
            active: Set(cmd.active),
            image_key: Set(None),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        txn.commit().await?;

        self.queries.get_employee_by_id(id).await?.ok_or_else(|| {
            Dbpg003Error::Internal("Failed to retrieve created employee".to_string())
        })
    }
}
