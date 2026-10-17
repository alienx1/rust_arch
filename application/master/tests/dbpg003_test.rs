//! Tests for DBPG003 (Employees) using a mock command set.
//! Located at `application/master/tests/dbpg003_test.rs`.

use application_master::features::db::dbpg003::{
    BranchOptionDto, CompanyOptionDto, CreateEmployeeCommand, Dbpg003Commands, Dbpg003Error,
    Dbpg003SummaryDto, DepartmentOptionDto, EmployeeDto, EmployeeFilterQuery, EmployeePageDto,
    PrefixOptionDto, UpdateEmployeeCommand,
};
use async_trait::async_trait;
use uuid::Uuid;

fn employee(department_id: Uuid, code: &str, first: &str, active: bool) -> EmployeeDto {
    EmployeeDto {
        id: Uuid::now_v7(),
        organization_id: Uuid::now_v7(),
        company_id: Uuid::now_v7(),
        company_code: "CMP01".to_string(),
        company_name: "บริษัท 1".to_string(),
        branch_id: Uuid::now_v7(),
        branch_code: "HO".to_string(),
        branch_name: "สำนักงานใหญ่".to_string(),
        department_id,
        department_code: "IT".to_string(),
        department_name: "ไอที".to_string(),
        person_name_id: Uuid::now_v7(),
        prefix_id: Uuid::now_v7(),
        prefix_name: "นาย".to_string(),
        code: code.to_string(),
        full_name: format!("นาย {first} ใจดี"),
        first_name_th: first.to_string(),
        middle_name_th: String::new(),
        last_name_th: "ใจดี".to_string(),
        nickname_th: String::new(),
        first_name_en: first.to_string(),
        middle_name_en: String::new(),
        last_name_en: "Jaidee".to_string(),
        nickname_en: String::new(),
        active,
        has_user_account: false,
        row_version: 1,
        image_url: None,
    }
}

/// Mock set that pages/filters like the real queries and validates like the real executor.
struct MockDbpg003Commands {
    employees: Vec<EmployeeDto>,
}

#[async_trait]
impl Dbpg003Commands for MockDbpg003Commands {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_branch_options(&self) -> Result<Vec<BranchOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_department_options(&self) -> Result<Vec<DepartmentOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }
    async fn list_prefixes(&self) -> Result<Vec<PrefixOptionDto>, Dbpg003Error> {
        Ok(vec![])
    }

    async fn list_employees(
        &self,
        filter: EmployeeFilterQuery,
    ) -> Result<EmployeePageDto, Dbpg003Error> {
        let mut rows = self.employees.clone();
        if let Some(d) = filter.department_id {
            rows.retain(|e| e.department_id == d);
        }
        if let Some(a) = filter.active {
            rows.retain(|e| e.active == a);
        }
        if let Some(q) = filter.search.as_deref().map(str::to_lowercase) {
            rows.retain(|e| e.code.to_lowercase().contains(&q) || e.full_name.contains(&q));
        }
        let limit = filter.limit.unwrap_or(25).clamp(1, 200);
        let page = filter.page.unwrap_or(1).max(1);
        let total = rows.len() as u64;
        let items = rows.into_iter().skip(((page - 1) * limit) as usize).take(limit as usize).collect();
        Ok(EmployeePageDto { items, total, page, limit })
    }

    async fn get_employee_by_id(&self, id: Uuid) -> Result<Option<EmployeeDto>, Dbpg003Error> {
        Ok(self.employees.iter().find(|e| e.id == id).cloned())
    }

    async fn get_summary(&self) -> Result<Dbpg003SummaryDto, Dbpg003Error> {
        Ok(Dbpg003SummaryDto {
            total_employees: self.employees.len() as u64,
            active_employees: self.employees.iter().filter(|e| e.active).count() as u64,
            with_user_account: self.employees.iter().filter(|e| e.has_user_account).count() as u64,
        })
    }

    async fn create_employee(
        &self,
        cmd: CreateEmployeeCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        let code = cmd.code.trim().to_uppercase();
        if code.is_empty() {
            return Err(Dbpg003Error::Validation("Employee code is required".to_string()));
        }
        if cmd.first_name_th.trim().is_empty() || cmd.last_name_th.trim().is_empty() {
            return Err(Dbpg003Error::Validation("Thai name is required".to_string()));
        }
        if self.employees.iter().any(|e| e.department_id == cmd.department_id && e.code == code) {
            return Err(Dbpg003Error::DuplicateCode(code));
        }
        Ok(employee(cmd.department_id, &code, &cmd.first_name_th, cmd.active))
    }

    async fn delete_employee(&self, id: Uuid) -> Result<(), Dbpg003Error> {
        if self.employees.iter().any(|e| e.id == id) {
            Ok(())
        } else {
            Err(Dbpg003Error::NotFound(id.to_string()))
        }
    }

    async fn update_employee(
        &self,
        id: Uuid,
        cmd: UpdateEmployeeCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<EmployeeDto, Dbpg003Error> {
        let existing = self
            .employees
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Dbpg003Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg003Error::VersionConflict);
        }
        let mut e = existing.clone();
        e.code = cmd.code.trim().to_uppercase();
        e.row_version += 1;
        Ok(e)
    }
}

fn create_cmd(department_id: Uuid, code: &str, first_th: &str) -> CreateEmployeeCommand {
    CreateEmployeeCommand {
        department_id,
        prefix_id: Uuid::now_v7(),
        code: code.to_string(),
        active: true,
        first_name_th: first_th.to_string(),
        middle_name_th: String::new(),
        last_name_th: "ใจดี".to_string(),
        nickname_th: String::new(),
        first_name_en: String::new(),
        middle_name_en: String::new(),
        last_name_en: String::new(),
        nickname_en: String::new(),
    }
}

fn fixture() -> (Uuid, MockDbpg003Commands) {
    let dept = Uuid::now_v7();
    let mut linked = employee(dept, "E003", "สมศรี", false);
    linked.has_user_account = true;
    (
        dept,
        MockDbpg003Commands {
            employees: vec![
                employee(dept, "E001", "สมชาย", true),
                employee(dept, "E002", "สมหญิง", true),
                linked,
                employee(Uuid::now_v7(), "X001", "อื่น", true),
            ],
        },
    )
}

#[tokio::test]
async fn test_list_employees_filters_and_pages() {
    let (dept, mock) = fixture();
    let page = mock
        .list_employees(EmployeeFilterQuery {
            department_id: Some(dept),
            limit: Some(2),
            page: Some(2),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.page, 2);

    let active = mock
        .list_employees(EmployeeFilterQuery {
            department_id: Some(dept),
            active: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(active.total, 2);
}

#[tokio::test]
async fn test_summary_counts_active_and_linked_accounts() {
    let (_, mock) = fixture();
    let s = mock.get_summary().await.unwrap();
    assert_eq!(s.total_employees, 4);
    assert_eq!(s.active_employees, 3);
    assert_eq!(s.with_user_account, 1);
}

#[tokio::test]
async fn test_create_employee_validates_and_rejects_duplicate_code() {
    let (dept, mock) = fixture();

    let created = mock.create_employee(create_cmd(dept, "e100", "สมปอง"), None).await.unwrap();
    assert_eq!(created.code, "E100");

    let no_name = mock.create_employee(create_cmd(dept, "E101", " "), None).await.unwrap_err();
    assert!(matches!(no_name, Dbpg003Error::Validation(_)));

    let dup = mock.create_employee(create_cmd(dept, "e001", "ซ้ำ"), None).await.unwrap_err();
    assert!(matches!(dup, Dbpg003Error::DuplicateCode(_)));
}

#[tokio::test]
async fn test_update_employee_checks_row_version() {
    let (dept, mock) = fixture();
    let target = mock.employees[0].clone();
    let cmd = |row_version: i64| UpdateEmployeeCommand {
        department_id: dept,
        prefix_id: target.prefix_id,
        code: "e001a".to_string(),
        active: true,
        first_name_th: "สมชาย".to_string(),
        middle_name_th: String::new(),
        last_name_th: "ใจดี".to_string(),
        nickname_th: String::new(),
        first_name_en: String::new(),
        middle_name_en: String::new(),
        last_name_en: String::new(),
        nickname_en: String::new(),
        row_version,
    };

    let updated = mock.update_employee(target.id, cmd(1), None).await.unwrap();
    assert_eq!(updated.code, "E001A");
    assert_eq!(updated.row_version, 2);

    let stale = mock.update_employee(target.id, cmd(5), None).await.unwrap_err();
    assert!(matches!(stale, Dbpg003Error::VersionConflict));
}
