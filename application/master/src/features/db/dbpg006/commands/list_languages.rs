//! DBPG006 command: list the active languages

use crate::features::db::dbpg006::errors::Dbpg006Error;
use crate::features::db::dbpg006::models::LanguageOptionDto;

use super::dbpg006_executor::Dbpg006CommandExecutor;

impl Dbpg006CommandExecutor {
    pub(super) async fn list_languages(&self) -> Result<Vec<LanguageOptionDto>, Dbpg006Error> {
        self.queries.list_languages().await
    }
}
