//! DBPG004 command: validate and create a lookup value with its labels in one transaction

use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use tracing::info;
use uuid::Uuid;

use crate::features::db::dbpg004::errors::Dbpg004Error;
use crate::features::db::dbpg004::models::{
    CreateLookupCommand, ENGLISH_LANGUAGE_ID, LookupDto, THAI_LANGUAGE_ID, db_master_lookup,
    is_protected_category,
};

use super::dbpg004_executor::Dbpg004CommandExecutor;
use super::lookup_label_upsert::upsert_label;
use super::lookup_rules::{normalize_category, normalize_code, required_label};

impl Dbpg004CommandExecutor {
    pub(super) async fn create_lookup(
        &self,
        cmd: CreateLookupCommand,
        actor_id: Option<Uuid>,
    ) -> Result<LookupDto, Dbpg004Error> {
        let category = normalize_category(&cmd.category)?;
        let code = normalize_code(&cmd.code)?;
        let name_th = required_label(&cmd.name_th)?;
        let name_en = match cmd.name_en.trim() {
            "" => name_th.clone(),
            en => en.to_string(),
        };

        if is_protected_category(&category) {
            return Err(Dbpg004Error::Validation(format!(
                "Category '{category}' is system-protected: values cannot be added"
            )));
        }

        // Case-insensitive, so `head` and `HEAD` cannot coexist
        let duplicate = db_master_lookup::Entity::find()
            .filter(db_master_lookup::Column::Category.eq(&category))
            .filter(
                Expr::expr(Func::upper(Expr::col(db_master_lookup::Column::Code))).eq(code.clone()),
            )
            .one(&self.db)
            .await?;
        if duplicate.is_some() {
            return Err(Dbpg004Error::DuplicateCode(format!(
                "Code '{code}' already exists in category '{category}'"
            )));
        }

        let id = Uuid::now_v7();
        info!("Creating new lookup: id={id}, category={category}, code={code}");

        let txn = self.db.begin().await?;
        db_master_lookup::ActiveModel {
            id: Set(id),
            category: Set(category),
            code: Set(code),
            active: Set(cmd.active),
            change_by: Set(actor_id),
            row_version: Set(1),
        }
        .insert(&txn)
        .await?;
        upsert_label(&txn, id, THAI_LANGUAGE_ID, name_th, actor_id, 1).await?;
        upsert_label(&txn, id, ENGLISH_LANGUAGE_ID, name_en, actor_id, 1).await?;
        txn.commit().await?;

        self.queries
            .get_lookup_by_id(id)
            .await?
            .ok_or_else(|| Dbpg004Error::Internal("Failed to retrieve created lookup".to_string()))
    }
}
