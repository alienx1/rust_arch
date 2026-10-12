//! Tests for DBPG004 (Reference Data): input normalization, protected categories and a mock
//! command set for list/create/update rules.
//! Located at `application/master/tests/dbpg004_test.rs`.

use application_master::features::db::dbpg004::{
    CreateLookupCommand, Dbpg004Commands, Dbpg004Error, Dbpg004SummaryDto, LookupCategoryDto,
    LookupDto, LookupFilterQuery, LookupPageDto, UpdateLookupCommand, is_protected_category,
    normalize_category, normalize_code,
};
use async_trait::async_trait;
use uuid::Uuid;

fn lookup(category: &str, code: &str, active: bool) -> LookupDto {
    LookupDto {
        id: Uuid::now_v7(),
        category: category.to_string(),
        code: code.to_string(),
        active,
        name_th: format!("ชื่อ {code}"),
        name_en: format!("Name {code}"),
        is_protected: is_protected_category(category),
        row_version: 1,
    }
}

struct MockDbpg004Commands {
    rows: Vec<LookupDto>,
}

#[async_trait]
impl Dbpg004Commands for MockDbpg004Commands {
    async fn list_categories(&self) -> Result<Vec<LookupCategoryDto>, Dbpg004Error> {
        let mut cats: Vec<LookupCategoryDto> = Vec::new();
        for r in &self.rows {
            match cats.iter_mut().find(|c| c.category == r.category) {
                Some(c) => {
                    c.total += 1;
                    c.active += u64::from(r.active);
                }
                None => cats.push(LookupCategoryDto {
                    category: r.category.clone(),
                    total: 1,
                    active: u64::from(r.active),
                    is_protected: r.is_protected,
                }),
            }
        }
        Ok(cats)
    }

    async fn list_lookups(&self, filter: LookupFilterQuery) -> Result<LookupPageDto, Dbpg004Error> {
        let mut rows = self.rows.clone();
        if let Some(c) = &filter.category {
            rows.retain(|r| &r.category == c);
        }
        if let Some(a) = filter.active {
            rows.retain(|r| r.active == a);
        }
        let limit = filter.limit.unwrap_or(25);
        let total = rows.len() as u64;
        Ok(LookupPageDto { items: rows.into_iter().take(limit as usize).collect(), total, page: 1, limit })
    }

    async fn get_lookup_by_id(&self, id: Uuid) -> Result<Option<LookupDto>, Dbpg004Error> {
        Ok(self.rows.iter().find(|r| r.id == id).cloned())
    }

    async fn get_summary(&self) -> Result<Dbpg004SummaryDto, Dbpg004Error> {
        let cats = self.list_categories().await?;
        Ok(Dbpg004SummaryDto {
            total_lookups: self.rows.len() as u64,
            active_lookups: self.rows.iter().filter(|r| r.active).count() as u64,
            total_categories: cats.len() as u64,
        })
    }

    async fn create_lookup(
        &self,
        cmd: CreateLookupCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        let category = normalize_category(&cmd.category)?;
        let code = normalize_code(&cmd.code)?;
        if is_protected_category(&category) {
            return Err(Dbpg004Error::Validation("protected".to_string()));
        }
        if self.rows.iter().any(|r| r.category == category && r.code == code) {
            return Err(Dbpg004Error::DuplicateCode(code));
        }
        Ok(lookup(&category, &code, cmd.active))
    }

    async fn delete_lookup(&self, id: Uuid) -> Result<(), Dbpg004Error> {
        let existing = self
            .rows
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| Dbpg004Error::NotFound(id.to_string()))?;
        if existing.is_protected {
            return Err(Dbpg004Error::Validation("protected value cannot be deleted".to_string()));
        }
        Ok(())
    }

    async fn update_lookup(
        &self,
        id: Uuid,
        cmd: UpdateLookupCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        let existing = self
            .rows
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| Dbpg004Error::NotFound(id.to_string()))?;
        if existing.row_version != cmd.row_version {
            return Err(Dbpg004Error::VersionConflict);
        }
        if existing.is_protected && existing.active && !cmd.active {
            return Err(Dbpg004Error::Validation("protected".to_string()));
        }
        let mut r = existing.clone();
        r.active = cmd.active;
        r.row_version += 1;
        Ok(r)
    }
}

fn fixture() -> MockDbpg004Commands {
    MockDbpg004Commands {
        rows: vec![
            lookup("gender", "MALE", true),
            lookup("gender", "FEMALE", true),
            lookup("gender", "OTHER", false),
            lookup("permission_type", "READ", true),
        ],
    }
}

fn create(category: &str, code: &str) -> CreateLookupCommand {
    CreateLookupCommand {
        category: category.to_string(),
        code: code.to_string(),
        active: true,
        name_th: "ชื่อ".to_string(),
        name_en: String::new(),
    }
}

#[test]
fn test_category_and_code_normalization() {
    assert_eq!(normalize_category("  Payment_Method ").unwrap(), "payment_method");
    assert!(normalize_category("has space").is_err());
    assert!(normalize_category("ไทย").is_err());
    assert!(normalize_category("  ").is_err());

    assert_eq!(normalize_code(" head ").unwrap(), "HEAD");
    assert!(normalize_code("").is_err());
    assert!(normalize_code(&"x".repeat(101)).is_err());
}

#[tokio::test]
async fn test_categories_count_rows_and_active() {
    let cats = fixture().list_categories().await.unwrap();
    let gender = cats.iter().find(|c| c.category == "gender").unwrap();
    assert_eq!((gender.total, gender.active), (3, 2));
    assert!(cats.iter().find(|c| c.category == "permission_type").unwrap().is_protected);

    let s = fixture().get_summary().await.unwrap();
    assert_eq!((s.total_lookups, s.active_lookups, s.total_categories), (4, 3, 2));
}

#[tokio::test]
async fn test_create_rejects_duplicates_and_protected_categories() {
    let m = fixture();
    let ok = m.create_lookup(create("gender", "unknown"), None).await.unwrap();
    assert_eq!(ok.code, "UNKNOWN");

    let dup = m.create_lookup(create("gender", "male"), None).await.unwrap_err();
    assert!(matches!(dup, Dbpg004Error::DuplicateCode(_)));

    let protected = m.create_lookup(create("permission_type", "DELETE"), None).await.unwrap_err();
    assert!(matches!(protected, Dbpg004Error::Validation(_)));
}

#[tokio::test]
async fn test_update_checks_version_and_protects_system_rows() {
    let m = fixture();
    let male = m.rows[0].clone();
    let read = m.rows[3].clone();
    let cmd = |active: bool, row_version: i64| UpdateLookupCommand {
        active,
        name_th: "ชื่อ".to_string(),
        name_en: String::new(),
        row_version,
    };

    assert_eq!(m.update_lookup(male.id, cmd(false, 1), None).await.unwrap().row_version, 2);
    assert!(matches!(
        m.update_lookup(male.id, cmd(true, 9), None).await.unwrap_err(),
        Dbpg004Error::VersionConflict
    ));
    // A system-core value can be relabelled but not switched off
    assert!(m.update_lookup(read.id, cmd(true, 1), None).await.is_ok());
    assert!(matches!(
        m.update_lookup(read.id, cmd(false, 1), None).await.unwrap_err(),
        Dbpg004Error::Validation(_)
    ));
}
