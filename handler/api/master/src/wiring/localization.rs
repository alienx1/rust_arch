//! Localization commands (dictionaries and API error templates, read through Valkey)

use application_master::features::localization::{I18nCommandExecutor, LocalizationCommands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let commands: Arc<dyn LocalizationCommands> = Arc::new(I18nCommandExecutor::new(cx.loc_queries.clone()));
    services.insert(commands);
}
