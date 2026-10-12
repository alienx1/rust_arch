//! Sea-ORM query execution against `db_label` for i18n dictionary.

use crate::features::localization::errors::LocalizationError;
use crate::features::localization::models::I18nDictionary;
use crate::features::localization::queries::LocalizationQueries;
use async_trait::async_trait;
use domain::entity::db_label;
use domain::entity::db_language;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

/// Executes queries on `db_label` table via Sea-ORM.
#[derive(Clone)]
pub struct DbLabelQueries {
    conn: DatabaseConnection,
}

impl DbLabelQueries {
    /// Create query runner with an active Sea-ORM DatabaseConnection.
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Access underlying database connection.
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

// Backward-compatible alias
pub type LocalizationQueryService = DbLabelQueries;

#[async_trait]
impl LocalizationQueries for DbLabelQueries {
    /// Fetch all UI dictionary labels for the given language ID (`db_language.id` / lid).
    /// Returns an `I18nDictionary` with keys formatted as `"<category>.<key>"` => `"<value>"`.
    async fn get_i18n_dictionary(
        &self,
        language_id: Uuid,
    ) -> Result<I18nDictionary, LocalizationError> {
        let rows = db_label::Entity::find()
            .filter(db_label::Column::LanguageId.eq(language_id))
            .all(&self.conn)
            .await
            .map_err(|e| LocalizationError::Database(e.to_string()))?;

        let mut dict = I18nDictionary::with_capacity(language_id, rows.len());
        for row in rows {
            let full_key = format!("{}.{}", row.category, row.key);
            dict.insert(full_key, row.value);
        }

        Ok(dict)
    }
    /// Look up `db_language.id` by code, ignoring case (`TH` == `th`).
    async fn get_language_id_by_code(&self, code: &str) -> Result<Option<Uuid>, LocalizationError> {
        let row = db_language::Entity::find()
            .filter(Expr::expr(Func::lower(Expr::col(db_language::Column::Code))).eq(code.trim().to_lowercase()))
            .one(&self.conn)
            .await
            .map_err(|e| LocalizationError::Database(e.to_string()))?;
        Ok(row.map(|r| r.id))
    }
}
