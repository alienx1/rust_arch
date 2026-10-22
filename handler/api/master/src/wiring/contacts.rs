//! Contacts of organizations, companies, branches and employees. An employee's default e-mail is
//! the account's sign-in e-mail, so it is kept in step at the identity provider.

use application_master::features::db::contacts::{ContactCommandExecutor, ContactCommands};
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let mut executor = ContactCommandExecutor::new(cx.db.clone());
    if let Some(k) = &cx.kratos {
        executor = executor.with_identity(Arc::new(k.clone()));
    }
    let commands: Arc<dyn ContactCommands> = Arc::new(executor);
    services.insert(commands);
}
