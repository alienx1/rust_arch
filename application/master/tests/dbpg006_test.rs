//! Tests for DBPG006 (i18n Labels): category / key normalization and a mock command set for the
//! entry rules (duplicate, not found, listing).
//! Located at `application/master/tests/dbpg006_test.rs`.

use application_master::features::db::dbpg006::{
    CreateLabelCommand, Dbpg006Commands, Dbpg006Error, LabelCategoryDto, LabelEntryDto,
    LabelFilterQuery, LabelPageDto, LabelValueDto, LabelValueInput, LanguageOptionDto,
    UpdateLabelCommand, normalize_label_category, normalize_label_key,
};
use async_trait::async_trait;
use uuid::Uuid;

#[test]
fn test_category_is_trimmed_and_restricted() {
    assert_eq!(normalize_label_category("  menu ").unwrap(), "menu");
    assert_eq!(normalize_label_category("Err-code_1.x").unwrap(), "Err-code_1.x");
    assert_eq!(normalize_label_category(&"a".repeat(100)).unwrap().len(), 100);

    for bad in ["", "   ", "has space", "slash/ed", "ไทย", "semi;colon"] {
        assert!(
            matches!(normalize_label_category(bad), Err(Dbpg006Error::Validation(_))),
            "{bad:?} must be refused"
        );
    }
    assert!(normalize_label_category(&"a".repeat(101)).is_err());
}

#[test]
fn test_key_is_trimmed_and_bounded() {
    assert_eq!(normalize_label_key("  column.thai_name ").unwrap(), "column.thai_name");
    // Keys may hold spaces and non-ASCII text
    assert_eq!(normalize_label_key("ปุ่ม บันทึก").unwrap(), "ปุ่ม บันทึก");
    assert_eq!(normalize_label_key(&"k".repeat(300)).unwrap().len(), 300);

    assert!(matches!(normalize_label_key(" "), Err(Dbpg006Error::Validation(_))));
    assert!(normalize_label_key(&"k".repeat(301)).is_err());
}

struct MockDbpg006Commands {
    entries: Vec<LabelEntryDto>,
}

fn entry(category: &str, key: &str, th: &str) -> LabelEntryDto {
    LabelEntryDto {
        category: category.to_string(),
        key: key.to_string(),
        values: vec![LabelValueDto {
            language_id: Uuid::now_v7(),
            language_code: "th".to_string(),
            value: th.to_string(),
        }],
        row_version: 1,
    }
}

fn input(value: &str) -> Vec<LabelValueInput> {
    vec![LabelValueInput { language_id: Uuid::now_v7(), value: value.to_string() }]
}

#[async_trait]
impl Dbpg006Commands for MockDbpg006Commands {
    async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        Ok(vec![])
    }

    async fn list_categories(&self) -> Result<Vec<LabelCategoryDto>, Dbpg006Error> {
        let mut cats: Vec<LabelCategoryDto> = Vec::new();
        for e in &self.entries {
            match cats.iter_mut().find(|c| c.category == e.category) {
                Some(c) => c.entries += 1,
                None => cats.push(LabelCategoryDto { category: e.category.clone(), entries: 1 }),
            }
        }
        Ok(cats)
    }

    async fn list_labels(&self, filter: LabelFilterQuery) -> Result<LabelPageDto, Dbpg006Error> {
        let mut items = self.entries.clone();
        if let Some(c) = &filter.category {
            items.retain(|e| &e.category == c);
        }
        let limit = filter.limit.unwrap_or(25);
        let total = items.len() as u64;
        Ok(LabelPageDto {
            items: items.into_iter().take(limit as usize).collect(),
            total,
            page: filter.page.unwrap_or(1),
            limit,
        })
    }

    async fn get_label(&self, category: &str, key: &str) -> Result<Option<LabelEntryDto>, Dbpg006Error> {
        Ok(self.entries.iter().find(|e| e.category == category && e.key == key).cloned())
    }

    async fn create_label(
        &self,
        cmd: CreateLabelCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error> {
        let category = normalize_label_category(&cmd.category)?;
        let key = normalize_label_key(&cmd.key)?;
        if self.entries.iter().any(|e| e.category == category && e.key == key) {
            return Err(Dbpg006Error::Duplicate(key));
        }
        Ok(entry(&category, &key, &cmd.values[0].value))
    }

    async fn update_label(
        &self,
        cmd: UpdateLabelCommand,
        _actor_id: Option<Uuid>,
    ) -> Result<LabelEntryDto, Dbpg006Error> {
        let category = normalize_label_category(&cmd.category)?;
        let key = normalize_label_key(&cmd.key)?;
        self.get_label(&category, &key)
            .await?
            .ok_or_else(|| Dbpg006Error::NotFound(key))
    }

    async fn delete_label(&self, category: &str, key: &str) -> Result<(), Dbpg006Error> {
        match self.get_label(category, key).await? {
            Some(_) => Ok(()),
            None => Err(Dbpg006Error::NotFound(key.to_string())),
        }
    }
}

fn fixture() -> MockDbpg006Commands {
    MockDbpg006Commands {
        entries: vec![
            entry("menu", "dashboard", "ภาพรวม"),
            entry("menu", "settings", "ตั้งค่า"),
            entry("error", "not_found", "ไม่พบข้อมูล"),
        ],
    }
}

#[tokio::test]
async fn test_categories_count_entries() {
    let cats = fixture().list_categories().await.unwrap();
    assert_eq!(cats.len(), 2);
    assert_eq!(cats.iter().find(|c| c.category == "menu").unwrap().entries, 2);
}

#[tokio::test]
async fn test_list_filters_by_category_and_reports_total() {
    let page = fixture()
        .list_labels(LabelFilterQuery { category: Some("menu".to_string()), limit: Some(1), ..Default::default() })
        .await
        .unwrap();
    assert_eq!((page.items.len(), page.total, page.limit), (1, 2, 1));
}

#[tokio::test]
async fn test_create_rejects_duplicates_and_invalid_input() {
    let cmds = fixture();
    let create = |category: &str, key: &str| CreateLabelCommand {
        category: category.to_string(),
        key: key.to_string(),
        values: input("x"),
    };

    assert!(matches!(
        cmds.create_label(create("menu", "dashboard"), None).await,
        Err(Dbpg006Error::Duplicate(_))
    ));
    assert!(matches!(
        cmds.create_label(create("bad cat", "k"), None).await,
        Err(Dbpg006Error::Validation(_))
    ));
    let created = cmds.create_label(create(" menu ", " reports "), None).await.unwrap();
    assert_eq!((created.category.as_str(), created.key.as_str()), ("menu", "reports"));
}

#[tokio::test]
async fn test_update_and_delete_need_an_existing_entry() {
    let cmds = fixture();
    let update = |key: &str| UpdateLabelCommand {
        category: "menu".to_string(),
        key: key.to_string(),
        values: input("y"),
        row_version: 1,
    };

    assert!(cmds.update_label(update("settings"), None).await.is_ok());
    assert!(matches!(cmds.update_label(update("missing"), None).await, Err(Dbpg006Error::NotFound(_))));
    assert!(cmds.delete_label("menu", "settings").await.is_ok());
    assert!(matches!(cmds.delete_label("menu", "missing").await, Err(Dbpg006Error::NotFound(_))));
}
