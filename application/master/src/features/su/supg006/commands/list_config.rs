//! SUPG006 read: every editable setting with its stored value

use std::collections::HashMap;

use super::supg006_executor::Supg006CommandExecutor;
use crate::features::su::supg006::errors::Supg006Error;
use crate::features::su::supg006::models::{CONFIG_SETTINGS, ConfigItemDto};

impl Supg006CommandExecutor {
    pub(super) async fn list_config(&self) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        let rows: HashMap<String, _> = self.queries.list_rows().await?.into_iter().map(|r| (r.key.clone(), r)).collect();
        Ok(CONFIG_SETTINGS
            .iter()
            .map(|s| {
                let row = rows.get(s.key);
                ConfigItemDto {
                    key: s.key.to_string(),
                    group: s.group,
                    kind: s.kind,
                    value: row.map(|r| r.value.clone()).unwrap_or_else(|| s.default_value.to_string()),
                    default_value: s.default_value.to_string(),
                    min: s.min,
                    max: s.max,
                    description: row.and_then(|r| r.description.clone()),
                    row_version: row.map(|r| r.row_version).unwrap_or(0),
                }
            })
            .collect())
    }
}
