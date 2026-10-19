//! SUPG006 write: validate the changed settings and save them in one transaction

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use uuid::Uuid;

use super::supg006_executor::Supg006CommandExecutor;
use crate::features::optimistic_update::update_versioned;
use crate::features::su::supg006::errors::Supg006Error;
use crate::features::su::supg006::models::{
    ConfigChange, ConfigItemDto, ConfigKind, KNOWN_PLACEHOLDERS, SaveConfigCommand, find_setting,
};
use domain::entity::su_configuration;

/// Every `{{name}}` in `text`
fn placeholders(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                found.push(after[..end].trim());
                rest = &after[end + 2..];
            }
            None => break,
        }
    }
    found
}

/// Checks one change against what its key may hold; returns the value to store (trimmed for
/// numbers and one-line texts, bodies as typed)
pub fn validate_change(change: &ConfigChange) -> Result<String, Supg006Error> {
    let setting = find_setting(&change.key)
        .ok_or_else(|| Supg006Error::Validation(format!("'{}' is not a setting of this program", change.key)))?;
    let bad = |m: String| Supg006Error::Validation(format!("{}: {m}", setting.key));
    match setting.kind {
        ConfigKind::Number => {
            let n: i64 = change.value.trim().parse().map_err(|_| bad("must be a whole number".to_string()))?;
            let (min, max) = (setting.min.unwrap_or(i64::MIN), setting.max.unwrap_or(i64::MAX));
            if n < min || n > max {
                return Err(bad(format!("must be between {min} and {max}")));
            }
            Ok(n.to_string())
        }
        ConfigKind::Flag => match change.value.trim().to_lowercase().as_str() {
            "1" | "true" => Ok("1".to_string()),
            "0" | "false" => Ok("0".to_string()),
            _ => Err(bad("must be on or off (1 / 0)".to_string())),
        },
        ConfigKind::Text | ConfigKind::Body => {
            let value = if setting.kind == ConfigKind::Text { change.value.trim().to_string() } else { change.value.clone() };
            if value.chars().count() > setting.max_len {
                return Err(bad(format!("must be at most {} characters", setting.max_len)));
            }
            if setting.kind == ConfigKind::Text && value.contains(['\n', '\r']) {
                return Err(bad("must be one line".to_string()));
            }
            if setting.category == "mail_template" {
                if let Some(unknown) = placeholders(&value).into_iter().find(|p| !KNOWN_PLACEHOLDERS.contains(p)) {
                    return Err(bad(format!(
                        "unknown placeholder {{{{{unknown}}}}} (use {})",
                        KNOWN_PLACEHOLDERS.iter().map(|p| format!("{{{{{p}}}}}")).collect::<Vec<_>>().join(", ")
                    )));
                }
                // A body without the link would mail people a password mail they cannot use; an
                // empty body is fine (the built-in text is used)
                if setting.kind == ConfigKind::Body && !value.trim().is_empty() && !placeholders(&value).contains(&"link") {
                    return Err(bad("must contain {{link}}, or the mail cannot be used to set a password".to_string()));
                }
            }
            Ok(value)
        }
    }
}

impl Supg006CommandExecutor {
    pub(super) async fn save_config(
        &self,
        cmd: SaveConfigCommand,
        actor_id: Option<Uuid>,
    ) -> Result<Vec<ConfigItemDto>, Supg006Error> {
        // Validate everything first: nothing is written when one value is wrong
        let mut checked = Vec::with_capacity(cmd.changes.len());
        for change in &cmd.changes {
            if checked.iter().any(|(c, _): &(&ConfigChange, String)| c.key == change.key) {
                return Err(Supg006Error::Validation(format!("{} is listed twice", change.key)));
            }
            checked.push((change, validate_change(change)?));
        }

        let txn = self.db.begin().await?;
        for (change, value) in checked {
            let setting = find_setting(&change.key).expect("validated above");
            let row = su_configuration::Entity::find()
                .filter(su_configuration::Column::Key.eq(setting.key))
                .one(&txn)
                .await?;
            match row {
                Some(row) => {
                    if row.value == value {
                        continue; // nothing changed: the version stays
                    }
                    let mut am: su_configuration::ActiveModel = row.clone().into();
                    am.value = Set(value);
                    am.change_by = Set(actor_id);
                    update_versioned(
                        &txn,
                        am,
                        (su_configuration::Column::Id, su_configuration::Column::RowVersion),
                        row.id,
                        change.row_version,
                    )
                    .await?;
                }
                None => {
                    // The row was never seeded: only a form that saw no row (version 0) may create it
                    if change.row_version != 0 {
                        return Err(Supg006Error::VersionConflict);
                    }
                    su_configuration::ActiveModel {
                        id: Set(Uuid::now_v7()),
                        category: Set(setting.category.to_string()),
                        key: Set(setting.key.to_string()),
                        value: Set(value),
                        description: Set(None),
                        change_by: Set(actor_id),
                        row_version: Set(1),
                    }
                    .insert(&txn)
                    .await?;
                }
            }
        }
        txn.commit().await?;
        self.list_config().await
    }
}
