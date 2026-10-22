//! Tests for DBPG002 (Branches & Departments) using mock queries/commands.
//! Located at `application/master/tests/dbpg002_test.rs`.

use application_master::features::db::dbpg002::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, CreateBranchCommand, PermissionGroupOptionDto,
    CreateDepartmentCommand, DepartmentDto, DepartmentFilterQuery, Dbpg002Commands, Dbpg002Error,
    Dbpg002Queries, Dbpg002SummaryDto, UpdateBranchCommand, UpdateDepartmentCommand,
};
use async_trait::async_trait;
use uuid::Uuid;

fn branch(company_id: Uuid, code: &str, head: bool, active: bool) -> BranchDto {
    BranchDto {
        id: Uuid::now_v7(),
        organization_id: Uuid::now_v7(),
        company_id,
        company_code: "CMP01".to_string(),
        company_name: "บริษัท 1".to_string(),
        code: code.to_string(),
        master_lookup_id: Uuid::now_v7(),
        branch_type_code: if head { "HEAD" } else { "BRANCH" }.to_string(),
        branch_type_name: if head { "สำนักงานใหญ่" } else { "สาขา" }.to_string(),
        is_headquarter: head,
        active,
        name_th: format!("สาขา {code}"),
        name_en: format!("Branch {code}"),
        department_count: 0,
        row_version: 1,
    }
}

fn department(branch: &BranchDto, code: &str, active: bool) -> DepartmentDto {
    DepartmentDto {
        id: Uuid::now_v7(),
        organization_id: branch.organization_id,
        company_id: branch.company_id,
        company_code: branch.company_code.clone(),
        company_name: branch.company_name.clone(),
        branch_id: branch.id,
        branch_code: branch.code.clone(),
        branch_name: branch.name_th.clone(),
        code: code.to_string(),
        active,
        name_th: format!("แผนก {code}"),
        name_en: format!("Department {code}"),
        default_permission_group_id: None,
        default_permission_group_code: None,
        row_version: 1,
    }
}

/// Mock set: filters like the real queries and validates like the real executor.
struct MockDbpg002Commands {
    branches: Vec<BranchDto>,
    departments: Vec<DepartmentDto>,
}

#[async_trait]
impl Dbpg002Queries for MockDbpg002Commands {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        Ok(vec![])
    }

    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        Ok(vec![])
    }

    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        Ok(vec![])
    }

    async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        let mut res = self.branches.clone();
        if let Some(c) = filter.company_id {
            res.retain(|b| b.company_id == c);
        }
        if let Some(a) = filter.active {
            res.retain(|b| b.active == a);
        }
        Ok(res)
    }

    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error> {
        Ok(self.branches.iter().find(|b| b.id == id).cloned())
    }

    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        let mut res = self.departments.clone();
        if let Some(b) = filter.branch_id {
            res.retain(|d| d.branch_id == b);
        }
        if let Some(a) = filter.active {
            res.retain(|d| d.active == a);
        }
        Ok(res)
    }

    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error> {
        Ok(self.departments.iter().find(|d| d.id == id).cloned())
    }

    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        Ok(Dbpg002SummaryDto {
            total_branches: self.branches.len() as u64,
            active_branches: self.branches.iter().filter(|b| b.active).count() as u64,
            total_departments: self.departments.len() as u64,
            active_departments: self.departments.iter().filter(|d| d.active).count() as u64,
        })
    }
}

#[async_trait]
impl Dbpg002Commands for MockDbpg002Commands {
    async fn list_company_options(&self) -> Result<Vec<CompanyOptionDto>, Dbpg002Error> {
        Dbpg002Queries::list_company_options(self).await
    }

    async fn list_permission_group_options(&self) -> Result<Vec<PermissionGroupOptionDto>, Dbpg002Error> {
        Dbpg002Queries::list_permission_group_options(self).await
    }

    async fn list_branch_types(&self) -> Result<Vec<BranchTypeLookupDto>, Dbpg002Error> {
        Dbpg002Queries::list_branch_types(self).await
    }

    async fn list_branches(
        &self,
        filter: BranchFilterQuery,
    ) -> Result<Vec<BranchDto>, Dbpg002Error> {
        Dbpg002Queries::list_branches(self, filter).await
    }

    async fn get_branch_by_id(&self, id: Uuid) -> Result<Option<BranchDto>, Dbpg002Error> {
        Dbpg002Queries::get_branch_by_id(self, id).await
    }

    async fn list_departments(
        &self,
        filter: DepartmentFilterQuery,
    ) -> Result<Vec<DepartmentDto>, Dbpg002Error> {
        Dbpg002Queries::list_departments(self, filter).await
    }

    async fn get_department_by_id(&self, id: Uuid) -> Result<Option<DepartmentDto>, Dbpg002Error> {
        Dbpg002Queries::get_department_by_id(self, id).await
    }

    async fn get_summary(&self) -> Result<Dbpg002SummaryDto, Dbpg002Error> {
        Dbpg002Queries::get_summary(self).await
    }

    async fn create_branch(
        &self,
        cmd: CreateBranchCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg002Error::Validation("Branch code is required".to_string()));
        }
        let mut b = branch(cmd.company_id, &cmd.code.trim().to_uppercase(), false, cmd.active);
        b.master_lookup_id = cmd.master_lookup_id;
        Ok(b)
    }

    async fn update_branch(
        &self,
        id: Uuid,
        cmd: UpdateBranchCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<BranchDto, Dbpg002Error> {
        let existing = self
            .branches
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg002Error::VersionConflict);
        }
        let mut b = existing.clone();
        b.code = cmd.code.trim().to_uppercase();
        b.row_version += 1;
        Ok(b)
    }

    async fn create_department(
        &self,
        cmd: CreateDepartmentCommand,
        _actor_id: Option<Uuid>,
        _may_assign_groups: bool,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        let parent = self
            .branches
            .iter()
            .find(|b| b.id == cmd.branch_id)
            .ok_or_else(|| Dbpg002Error::NotFound(cmd.branch_id.to_string()))?;
        let code = cmd.code.trim().to_uppercase();
        if self.departments.iter().any(|d| d.branch_id == parent.id && d.code == code) {
            return Err(Dbpg002Error::DuplicateCode(code));
        }
        Ok(department(parent, &code, cmd.active))
    }

    async fn update_department(
        &self,
        id: Uuid,
        cmd: UpdateDepartmentCommand,
        _actor_id: Option<Uuid>,
        _may_assign_groups: bool,
    ) -> Result<DepartmentDto, Dbpg002Error> {
        let existing = self
            .departments
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg002Error::VersionConflict);
        }
        let mut d = existing.clone();
        d.branch_id = cmd.branch_id;
        d.code = cmd.code.trim().to_uppercase();
        d.row_version += 1;
        Ok(d)
    }

    async fn delete_branch(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        let b = self
            .branches
            .iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))?;
        if b.is_headquarter {
            return Err(Dbpg002Error::Validation("The headquarter branch cannot be deleted".into()));
        }
        if self.departments.iter().any(|d| d.branch_id == id) {
            return Err(Dbpg002Error::InUse("still referenced".to_string()));
        }
        Ok(())
    }

    async fn delete_department(&self, id: Uuid) -> Result<(), Dbpg002Error> {
        self.departments
            .iter()
            .find(|d| d.id == id)
            .map(|_| ())
            .ok_or_else(|| Dbpg002Error::NotFound(id.to_string()))
    }
}

fn fixture() -> (Uuid, MockDbpg002Commands) {
    let company = Uuid::now_v7();
    let hq = branch(company, "HO", true, true);
    let b1 = branch(company, "B001", false, false);
    let other = branch(Uuid::now_v7(), "X01", false, true);
    let departments = vec![department(&hq, "IT", true), department(&hq, "HR", false)];
    (
        company,
        MockDbpg002Commands { branches: vec![hq, b1, other], departments },
    )
}

#[tokio::test]
async fn test_filter_branches_by_company_and_status() {
    let (company, mock) = fixture();
    let of_company = Dbpg002Commands::list_branches(
        &mock,
        BranchFilterQuery { company_id: Some(company), ..Default::default() },
    )
    .await
    .unwrap();
    assert_eq!(of_company.len(), 2);
    assert!(of_company.iter().any(|b| b.is_headquarter && b.code == "HO"));

    let active = Dbpg002Commands::list_branches(
        &mock,
        BranchFilterQuery { company_id: Some(company), active: Some(true), search: None },
    )
    .await
    .unwrap();
    assert_eq!(active.len(), 1);
}

#[tokio::test]
async fn test_filter_departments_by_branch_and_summary() {
    let (_, mock) = fixture();
    let hq_id = mock.branches[0].id;
    let depts = Dbpg002Commands::list_departments(
        &mock,
        DepartmentFilterQuery { branch_id: Some(hq_id), active: Some(true), ..Default::default() },
    )
    .await
    .unwrap();
    assert_eq!(depts.len(), 1);
    assert_eq!(depts[0].code, "IT");

    let summary = Dbpg002Commands::get_summary(&mock).await.unwrap();
    assert_eq!(summary.total_branches, 3);
    assert_eq!(summary.active_branches, 2);
    assert_eq!(summary.total_departments, 2);
    assert_eq!(summary.active_departments, 1);
}

#[tokio::test]
async fn test_create_department_uppercases_and_rejects_duplicate() {
    let (_, mock) = fixture();
    let hq_id = mock.branches[0].id;
    let cmd = |code: &str| CreateDepartmentCommand {
        branch_id: hq_id,
        code: code.to_string(),
        active: true,
        name_th: "การเงิน".to_string(),
        name_en: "Finance".to_string(),
        default_permission_group_id: None,
    };

    let created = mock.create_department(cmd("acc"), None, true).await.unwrap();
    assert_eq!(created.code, "ACC");
    assert_eq!(created.branch_id, hq_id);

    let dup = mock.create_department(cmd("it"), None, true).await.unwrap_err();
    assert!(matches!(dup, Dbpg002Error::DuplicateCode(_)));
}

#[tokio::test]
async fn test_update_department_checks_row_version() {
    let (_, mock) = fixture();
    let dept = mock.departments[0].clone();
    let cmd = |row_version: i64| UpdateDepartmentCommand {
        branch_id: dept.branch_id,
        code: "it2".to_string(),
        active: true,
        name_th: "ไอที".to_string(),
        name_en: "IT".to_string(),
        default_permission_group_id: None,
        row_version,
    };

    let updated = mock.update_department(dept.id, cmd(1), None, true).await.unwrap();
    assert_eq!(updated.code, "IT2");
    assert_eq!(updated.row_version, 2);

    let stale = mock.update_department(dept.id, cmd(9), None, true).await.unwrap_err();
    assert!(matches!(stale, Dbpg002Error::VersionConflict));
}

#[tokio::test]
async fn test_create_branch_rejects_empty_code() {
    let (company, mock) = fixture();
    let err = mock
        .create_branch(
            CreateBranchCommand {
                company_id: company,
                code: " ".to_string(),
                master_lookup_id: Uuid::now_v7(),
                active: true,
                name_th: "สาขา".to_string(),
                name_en: String::new(),
            },
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Dbpg002Error::Validation(_)));
}

#[tokio::test]
async fn test_delete_branch_rules() {
    let (_, mock) = fixture();
    let (hq, b1) = (mock.branches[0].id, mock.branches[1].id);

    assert!(matches!(
        mock.delete_branch(hq).await.unwrap_err(),
        Dbpg002Error::Validation(_)
    ));
    mock.delete_branch(b1).await.unwrap();
    assert!(matches!(
        mock.delete_branch(Uuid::now_v7()).await.unwrap_err(),
        Dbpg002Error::NotFound(_)
    ));
}

#[tokio::test]
async fn test_delete_department_missing_is_not_found() {
    let (_, mock) = fixture();
    mock.delete_department(mock.departments[0].id).await.unwrap();
    assert!(matches!(
        mock.delete_department(Uuid::now_v7()).await.unwrap_err(),
        Dbpg002Error::NotFound(_)
    ));
}
