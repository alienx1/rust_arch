//! SUPG002 menu tree for an account (delegates to `Supg002Queries`)

use super::supg002_executor::Supg002CommandExecutor;
use crate::features::su::supg002::errors::Supg002Error;
use crate::features::su::supg002::models::MenuNodeDto;

impl Supg002CommandExecutor {
    pub(super) async fn get_menu_tree(&self, username: &str) -> Result<Vec<MenuNodeDto>, Supg002Error> {
        self.queries.menu_tree(username).await
    }
}
