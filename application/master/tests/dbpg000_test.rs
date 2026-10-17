//! Tests for DBPG000 (Organizations & Companies list) using mock queries/commands.
//! Located at `application/master/tests/dbpg000_test.rs`.

use application_master::features::db::dbpg000::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, Dbpg000Commands, Dbpg000Error,
    Dbpg000Queries, OrganizationDto, OrganizationSummaryDto,
};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Mock queries implementation for isolated testing
struct MockDbpg000Queries {
    organizations: Vec<OrganizationDto>,
    companies: Vec<CompanyDto>,
}

#[async_trait]
impl Dbpg000Queries for MockDbpg000Queries {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        Ok(self.organizations.clone())
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        Ok(self.organizations.iter().find(|o| o.id == id).cloned())
    }

    async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        let mut res = self.companies.clone();
        if let Some(org_id) = filter.organization_id {
            res.retain(|c| c.organization_id == org_id);
        }
        if let Some(active) = filter.active {
            res.retain(|c| c.active == active);
        }
        if let Some(q) = filter.search {
            let lower = q.to_lowercase();
            res.retain(|c| {
                c.code.to_lowercase().contains(&lower)
                    || c.name_th.to_lowercase().contains(&lower)
                    || c.name_en.to_lowercase().contains(&lower)
            });
        }
        Ok(res)
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error> {
        Ok(self.companies.iter().find(|c| c.id == id).cloned())
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        Ok(vec![
            CompanyTypeLookupDto {
                id: Uuid::now_v7(),
                code: "PRIVATE".to_string(),
                name_th: "บริษัทจำกัด".to_string(),
                name_en: "Company Limited".to_string(),
            },
            CompanyTypeLookupDto {
                id: Uuid::now_v7(),
                code: "PUBLIC".to_string(),
                name_th: "บริษัทมหาชนจำกัด".to_string(),
                name_en: "Public Limited Company".to_string(),
            },
        ])
    }

    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        let active_orgs = self.organizations.iter().filter(|o| o.active).count() as u64;
        let active_comps = self.companies.iter().filter(|c| c.active).count() as u64;
        Ok(OrganizationSummaryDto {
            total_organizations: self.organizations.len() as u64,
            active_organizations: active_orgs,
            total_companies: self.companies.len() as u64,
            active_companies: active_comps,
        })
    }
}

/// Simple test mock implementation of Dbpg000Commands
struct MockDbpg000Commands {
    queries: Arc<MockDbpg000Queries>,
}

#[async_trait]
impl Dbpg000Commands for MockDbpg000Commands {
    async fn list_organizations(&self) -> Result<Vec<OrganizationDto>, Dbpg000Error> {
        self.queries.list_organizations().await
    }

    async fn get_organization_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<OrganizationDto>, Dbpg000Error> {
        self.queries.get_organization_by_id(id).await
    }

    async fn list_companies(
        &self,
        filter: CompanyFilterQuery,
    ) -> Result<Vec<CompanyDto>, Dbpg000Error> {
        self.queries.list_companies(filter).await
    }

    async fn get_company_by_id(&self, id: Uuid) -> Result<Option<CompanyDto>, Dbpg000Error> {
        self.queries.get_company_by_id(id).await
    }

    async fn list_company_types(&self) -> Result<Vec<CompanyTypeLookupDto>, Dbpg000Error> {
        self.queries.list_company_types().await
    }

    async fn get_summary(&self) -> Result<OrganizationSummaryDto, Dbpg000Error> {
        self.queries.get_summary().await
    }
}

#[tokio::test]
async fn test_list_organizations() {
    let org_id = Uuid::now_v7();
    let mock = MockDbpg000Queries {
        organizations: vec![OrganizationDto {
            id: org_id,
            code: "ARCH".to_string(),
            registration_number: "0000000000000".to_string(),
            active: true,
            name_th: "องค์กร ARCH".to_string(),
            name_en: "ARCH Organization".to_string(),
            company_count: 1,
            row_version: 1,
            image_url: None,
        }],
        companies: vec![],
    };

    let orgs = mock.list_organizations().await.unwrap();
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0].code, "ARCH");
    assert_eq!(orgs[0].name_th, "องค์กร ARCH");
}

#[tokio::test]
async fn test_filter_companies_by_organization() {
    let org1_id = Uuid::now_v7();
    let org2_id = Uuid::now_v7();

    let comp1 = CompanyDto {
        id: Uuid::now_v7(),
        organization_id: org1_id,
        organization_code: "ORG1".to_string(),
        organization_name: "Organization 1".to_string(),
        code: "CMP01".to_string(),
        master_lookup_id: Uuid::now_v7(),
        company_type_name: "บริษัทจำกัด".to_string(),
        active: true,
        name_th: "บริษัท 1".to_string(),
        name_en: "Company 1".to_string(),
        row_version: 1,
        image_url: None,
    };
    let comp2 = CompanyDto {
        id: Uuid::now_v7(),
        organization_id: org2_id,
        organization_code: "ORG2".to_string(),
        organization_name: "Organization 2".to_string(),
        code: "CMP02".to_string(),
        master_lookup_id: Uuid::now_v7(),
        company_type_name: "บริษัทจำกัด".to_string(),
        active: true,
        name_th: "บริษัท 2".to_string(),
        name_en: "Company 2".to_string(),
        row_version: 1,
        image_url: None,
    };

    let mock = MockDbpg000Queries {
        organizations: vec![],
        companies: vec![comp1, comp2],
    };

    let filtered = mock
        .list_companies(CompanyFilterQuery {
            organization_id: Some(org1_id),
            search: None,
            active: None,
            page: None,
            limit: None,
        })
        .await
        .unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].code, "CMP01");
}

#[tokio::test]
async fn test_commands_summary_delegates_to_queries() {
    let queries = Arc::new(MockDbpg000Queries {
        organizations: vec![],
        companies: vec![],
    });
    let commands = MockDbpg000Commands { queries };

    let summary = commands.get_summary().await.unwrap();
    assert_eq!(summary.total_organizations, 0);
    assert_eq!(summary.total_companies, 0);
}
