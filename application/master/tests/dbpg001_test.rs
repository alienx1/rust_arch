//! Tests for DBPG001 (Organization & Company create/update) using a mock command set.
//! Located at `application/master/tests/dbpg001_test.rs`.

use application_master::features::db::dbpg001::{
    CompanyDto, CompanyTypeLookupDto, CreateCompanyCommand, CreateOrganizationCommand,
    Dbpg001Commands, Dbpg001Error, OrganizationDto, UpdateCompanyCommand,
    UpdateOrganizationCommand,
};
use async_trait::async_trait;
use uuid::Uuid;

/// Mock command set: validates like the real executor and echoes the written record.
struct MockDbpg001Commands;

#[async_trait]
impl Dbpg001Commands for MockDbpg001Commands {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg001Error> {
        Ok(vec![])
    }

    async fn get_organization_by_id(
        &self,
        _id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg001Error> {
        Ok(None)
    }

    async fn get_company_by_id(&self, _id: Uuid) -> Result<Option<CompanyDto>, Dbpg001Error> {
        Ok(None)
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg001Error> {
        Ok(vec![])
    }

    async fn create_organization(
        &self,
        cmd: CreateOrganizationCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg001Error::Validation("Code required".to_string()));
        }
        Ok(OrganizationDto {
            id: Uuid::now_v7(),
            code: cmd.code.trim().to_uppercase(),
            registration_number: cmd.registration_number,
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            company_count: 0,
            row_version: 1,
            image_url: None,
        })
    }

    async fn update_organization(
        &self,
        id: Uuid,
        cmd: UpdateOrganizationCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<OrganizationDto, Dbpg001Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg001Error::Validation("Code required".to_string()));
        }
        Ok(OrganizationDto {
            id,
            code: cmd.code.trim().to_uppercase(),
            registration_number: cmd.registration_number,
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            company_count: 0,
            row_version: cmd.row_version + 1,
            image_url: None,
        })
    }

    async fn create_company(
        &self,
        cmd: CreateCompanyCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg001Error::Validation("Code required".to_string()));
        }
        Ok(CompanyDto {
            id: Uuid::now_v7(),
            organization_id: cmd.organization_id,
            organization_code: "ARCH".to_string(),
            organization_name: "องค์กร ARCH".to_string(),
            code: cmd.code.trim().to_uppercase(),
            master_lookup_id: cmd.master_lookup_id,
            company_type_name: "บริษัทจำกัด".to_string(),
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            row_version: 1,
            image_url: None,
        })
    }

    async fn update_company(
        &self,
        id: Uuid,
        cmd: UpdateCompanyCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<CompanyDto, Dbpg001Error> {
        if cmd.code.trim().is_empty() {
            return Err(Dbpg001Error::Validation("Code required".to_string()));
        }
        Ok(CompanyDto {
            id,
            organization_id: cmd.organization_id,
            organization_code: "ARCH".to_string(),
            organization_name: "องค์กร ARCH".to_string(),
            code: cmd.code.trim().to_uppercase(),
            master_lookup_id: cmd.master_lookup_id,
            company_type_name: "บริษัทจำกัด".to_string(),
            active: cmd.active,
            name_th: cmd.name_th,
            name_en: cmd.name_en,
            row_version: cmd.row_version + 1,
            image_url: None,
        })
    }

    async fn delete_organization(&self, _id: Uuid) -> Result<(), Dbpg001Error> {
        Ok(())
    }

    async fn delete_company(&self, _id: Uuid) -> Result<(), Dbpg001Error> {
        Ok(())
    }
}

#[tokio::test]
async fn test_create_organization_command() {
    let commands = MockDbpg001Commands;

    let cmd = CreateOrganizationCommand {
        code: "neworg".to_string(),
        registration_number: "1234567890123".to_string(),
        active: true,
        name_th: "องค์กรใหม่".to_string(),
        name_en: "New Organization".to_string(),
    };

    let res = commands.create_organization(cmd, None).await.unwrap();
    assert_eq!(res.code, "NEWORG");
    assert_eq!(res.name_th, "องค์กรใหม่");
    assert_eq!(res.row_version, 1);
}

#[tokio::test]
async fn test_update_organization_bumps_row_version() {
    let id = Uuid::now_v7();
    let cmd = UpdateOrganizationCommand {
        code: "arch".to_string(),
        registration_number: "0000000000000".to_string(),
        active: true,
        name_th: "องค์กร ARCH".to_string(),
        name_en: "ARCH Organization".to_string(),
        row_version: 3,
    };

    let res = MockDbpg001Commands.update_organization(id, cmd, None).await.unwrap();
    assert_eq!(res.id, id);
    assert_eq!(res.code, "ARCH");
    assert_eq!(res.row_version, 4);
}

#[tokio::test]
async fn test_create_company_rejects_empty_code() {
    let cmd = CreateCompanyCommand {
        organization_id: Uuid::now_v7(),
        code: "  ".to_string(),
        master_lookup_id: Uuid::now_v7(),
        active: true,
        name_th: "บริษัท".to_string(),
        name_en: "Company".to_string(),
    };

    let err = MockDbpg001Commands.create_company(cmd, None).await.unwrap_err();
    assert!(matches!(err, Dbpg001Error::Validation(_)));
}
