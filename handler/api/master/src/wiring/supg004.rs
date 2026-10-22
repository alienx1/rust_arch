//! SUPG004 (user accounts) commands: accounts are created at the identity provider (Kratos) too,
//! and a new account is mailed its set-password link

use application_core::traits::IdentityProvisioningPort;
use application_master::features::su::account_security::AccountSecurityCommands;
use application_master::features::su::supg004::{DbSupg004Queries, Supg004CommandExecutor, Supg004Commands, Supg004Queries};
use application_master::features::su::supg005::Supg005Commands;
use std::sync::Arc;

use super::WiringContext;
use crate::services::ServicesBuilder;

pub fn register(cx: &WiringContext, services: &mut ServicesBuilder) {
    let queries: Arc<dyn Supg004Queries> = Arc::new(DbSupg004Queries::new(cx.db.clone()));
    let identity: Option<Arc<dyn IdentityProvisioningPort>> =
        cx.kratos.clone().map(|k| Arc::new(k) as Arc<dyn IdentityProvisioningPort>);
    let commands: Arc<dyn Supg004Commands> = Arc::new(
        Supg004CommandExecutor::new(queries, cx.db.clone(), identity, services.require::<dyn Supg005Commands>())
            .with_cache(cx.caches.clone())
            .with_account_security(services.require::<dyn AccountSecurityCommands>()),
    );
    services.insert(commands);
}
