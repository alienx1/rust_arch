//! SUPG002 menu tree node DTO

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One node of the menu tree: a folder (has children and usually no `path`) or a leaf program
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuNodeDto {
    pub id: Uuid,
    /// Program code, e.g. `DBPG003`
    pub code: String,
    /// SPA route of the program; `None` for a folder without a page of its own
    pub path: Option<String>,
    /// Icon name from the catalogue, e.g. `ApartmentOutlined`
    pub icon: Option<String>,
    pub label_th: String,
    pub label_en: String,
    pub sort_order: i32,
    pub children: Vec<MenuNodeDto>,
}
