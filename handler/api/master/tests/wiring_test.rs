//! The wiring: every feature's command interface is registered by `wiring::register_all`, so the
//! route handlers' `State<Arc<dyn ...>>` resolves. A feature that has a router but was not wired
//! would otherwise show up as a panic on its first request.
//! Located at `handler/api/master/tests/wiring_test.rs`.

use std::sync::Arc;

use api_master::services::{Services, ServicesBuilder};
use api_master::wiring::{WiringContext, register_all};
use application_core::traits::{NoopAuditLog, ReadCaches};
use application_master::features::al::alpg000::Alpg000Commands;
use application_master::features::al::alpg001::Alpg001Commands;
use application_master::features::dashboard::DashboardCommands;
use application_master::features::db::contacts::ContactCommands;
use application_master::features::db::dbpg000::Dbpg000Commands;
use application_master::features::db::dbpg001::Dbpg001Commands;
use application_master::features::db::dbpg002::Dbpg002Commands;
use application_master::features::db::dbpg003::Dbpg003Commands;
use application_master::features::db::dbpg004::Dbpg004Commands;
use application_master::features::db::dbpg005::Dbpg005Commands;
use application_master::features::db::dbpg006::Dbpg006Commands;
use application_master::features::localization::{DbLabelQueries, LocalizationCommands};
use application_master::features::su::access_control::AccessControl;
use application_master::features::su::account_security::AccountSecurityCommands;
use application_master::features::su::supg001::Supg001Commands;
use application_master::features::su::supg002::Supg002Commands;
use application_master::features::su::supg003::Supg003Commands;
use application_master::features::su::supg004::Supg004Commands;
use application_master::features::su::supg005::Supg005Commands;
use application_master::features::su::supg006::Supg006Commands;
use sea_orm::{DatabaseBackend, MockDatabase};

async fn services() -> Services {
    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    register_all(&WiringContext {
        db: db.clone(),
        cache: None,
        caches: ReadCaches::disabled(),
        kratos: None,
        audit_log: Arc::new(NoopAuditLog),
        broker: None,
        image_content: Arc::new(infrastructure::S3ContentService::new(infrastructure::s3_client_from_env().await)),
        image_bucket: "test-bucket".to_string(),
        loc_queries: Arc::new(DbLabelQueries::new(db)),
    })
}

#[tokio::test]
async fn test_every_feature_with_a_router_is_registered() {
    let s = services().await;
    // One line per feature that a router asks for: add yours when you wire a new program
    assert!(s.has::<dyn LocalizationCommands>(), "LocalizationCommands");
    assert!(s.has::<dyn DashboardCommands>(), "DashboardCommands");
    assert!(s.has::<dyn AccessControl>(), "AccessControl");
    assert!(s.has::<dyn AccountSecurityCommands>(), "AccountSecurityCommands");
    assert!(s.has::<dyn ContactCommands>(), "ContactCommands");
    assert!(s.has::<dyn Dbpg000Commands>(), "Dbpg000Commands");
    assert!(s.has::<dyn Dbpg001Commands>(), "Dbpg001Commands");
    assert!(s.has::<dyn Dbpg002Commands>(), "Dbpg002Commands");
    assert!(s.has::<dyn Dbpg003Commands>(), "Dbpg003Commands");
    assert!(s.has::<dyn Dbpg004Commands>(), "Dbpg004Commands");
    assert!(s.has::<dyn Dbpg005Commands>(), "Dbpg005Commands");
    assert!(s.has::<dyn Dbpg006Commands>(), "Dbpg006Commands");
    assert!(s.has::<dyn Supg001Commands>(), "Supg001Commands");
    assert!(s.has::<dyn Supg002Commands>(), "Supg002Commands");
    assert!(s.has::<dyn Supg003Commands>(), "Supg003Commands");
    assert!(s.has::<dyn Supg004Commands>(), "Supg004Commands");
    assert!(s.has::<dyn Supg005Commands>(), "Supg005Commands");
    assert!(s.has::<dyn Supg006Commands>(), "Supg006Commands");
    assert!(s.has::<dyn Alpg000Commands>(), "Alpg000Commands");
    assert!(s.has::<dyn Alpg001Commands>(), "Alpg001Commands");
}

trait Greeter: Send + Sync {
    fn hello(&self) -> &'static str;
}
struct English;
impl Greeter for English {
    fn hello(&self) -> &'static str {
        "hello"
    }
}
trait Other: Send + Sync {}

#[test]
fn test_the_registry_finds_a_service_by_its_type_only() {
    let mut b = ServicesBuilder::default();
    assert!(b.get::<dyn Greeter>().is_none());
    b.insert::<dyn Greeter>(Arc::new(English));
    assert_eq!(b.require::<dyn Greeter>().hello(), "hello", "a feature can build on one registered earlier");
    let s = b.build();
    assert_eq!(s.require::<dyn Greeter>().hello(), "hello");
    assert!(s.has::<dyn Greeter>() && !s.has::<dyn Other>(), "another type is another service");
    assert!(s.get::<dyn Other>().is_none());
}

#[test]
#[should_panic(expected = "was never registered")]
fn test_asking_for_a_service_that_was_never_wired_names_it() {
    ServicesBuilder::default().build().require::<dyn Greeter>();
}

#[test]
#[should_panic(expected = "wire it before the features that need it")]
fn test_a_feature_wired_before_the_one_it_builds_on_names_the_missing_one() {
    ServicesBuilder::default().require::<dyn Greeter>();
}
