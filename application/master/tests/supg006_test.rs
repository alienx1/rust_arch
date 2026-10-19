//! SUPG006: what each setting may hold, and the page's view of the stored rows.
//! Located at `application/master/tests/supg006_test.rs`.

use std::sync::Arc;

use application_master::features::su::supg006::{
    CONFIG_SETTINGS, ConfigChange, ConfigGroup, DbSupg006Queries, Supg006CommandExecutor, Supg006Commands, Supg006Error,
    Supg006Queries, validate_change,
};
use async_trait::async_trait;
use domain::entity::su_configuration;
use sea_orm::{DatabaseBackend, MockDatabase};
use uuid::Uuid;

fn change(key: &str, value: &str) -> ConfigChange {
    ConfigChange { key: key.to_string(), value: value.to_string(), row_version: 1 }
}

fn message(r: Result<String, Supg006Error>) -> String {
    match r {
        Err(Supg006Error::Validation(m)) => m,
        other => panic!("expected a validation error, got {other:?}"),
    }
}

#[test]
fn test_numbers_must_be_whole_and_in_range() {
    assert_eq!(validate_change(&change("security.login.max_failed_attempts", " 7 ")).unwrap(), "7");
    assert_eq!(validate_change(&change("security.login.max_failed_attempts", "0")).unwrap(), "0", "0 switches the lock off");
    assert!(message(validate_change(&change("security.login.max_failed_attempts", "101"))).contains("between 0 and 100"));
    assert!(message(validate_change(&change("security.login.lock_minutes", "0"))).contains("between 1"), "a lock of 0 minutes is no lock");
    assert!(message(validate_change(&change("security.login.lock_minutes", "ten"))).contains("whole number"));
    assert!(message(validate_change(&change("security.login.attempt_window_minutes", "-5"))).contains("between 1"));
}

#[test]
fn test_password_policy_settings_are_numbers_and_flags() {
    assert_eq!(validate_change(&change("security.password.min_length", "12")).unwrap(), "12");
    assert!(message(validate_change(&change("security.password.min_length", "7"))).contains("between 8 and 128"), "Kratos refuses under 8");
    assert!(message(validate_change(&change("security.password.min_length", "129"))).contains("between 8 and 128"));
    for (typed, stored) in [("1", "1"), ("0", "0"), (" TRUE ", "1"), ("false", "0")] {
        assert_eq!(validate_change(&change("security.password.require_symbol", typed)).unwrap(), stored, "{typed}");
    }
    assert!(message(validate_change(&change("security.password.require_digit", "yes"))).contains("on or off"));
    assert!(message(validate_change(&change("security.password.require_digit", "2"))).contains("on or off"));
}

#[test]
fn test_only_listed_keys_can_be_written() {
    assert!(message(validate_change(&change("mail.smtp_url", "smtp://x"))).contains("not a setting"), "the SMTP password stays out");
    assert!(message(validate_change(&change("security.anything", "1"))).contains("not a setting"));
}

#[test]
fn test_templates_accept_only_known_placeholders() {
    let ok = "<p>{{username}} {{ link }} {{hours}} {{app_name}}</p>";
    assert_eq!(validate_change(&change("mail.template.reset.html", ok)).unwrap(), ok, "bodies are stored as typed");
    let err = message(validate_change(&change("mail.template.reset.html", "<a href=\"{{link}}\">{{password}}</a>")));
    assert!(err.contains("unknown placeholder {{password}}"), "{err}");
}

#[test]
fn test_a_body_needs_the_link_unless_it_is_empty() {
    let err = message(validate_change(&change("mail.template.setup.text", "Hello {{username}}")));
    assert!(err.contains("{{link}}"), "{err}");
    assert!(message(validate_change(&change("mail.template.setup.html", "<p>no link</p>"))).contains("{{link}}"));
    assert_eq!(validate_change(&change("mail.template.setup.text", "  ")).unwrap(), "  ", "empty = the built-in text");
    // A subject has no link to carry
    assert_eq!(validate_change(&change("mail.template.setup.subject", "Hi {{username}}")).unwrap(), "Hi {{username}}");
}

#[test]
fn test_text_settings_are_one_trimmed_line_of_limited_length() {
    assert_eq!(validate_change(&change("mail.template.app_name", "  Arch  ")).unwrap(), "Arch");
    assert!(message(validate_change(&change("mail.template.app_name", "a\nb"))).contains("one line"));
    assert!(message(validate_change(&change("mail.template.reset.subject", &"x".repeat(201)))).contains("at most 200"));
    assert!(message(validate_change(&change("mail.template.reset.text", &format!("{{{{link}}}}{}", "x".repeat(20_000))))).contains("at most"));
}

struct Rows(Vec<su_configuration::Model>);

#[async_trait]
impl Supg006Queries for Rows {
    async fn list_rows(&self) -> Result<Vec<su_configuration::Model>, Supg006Error> {
        Ok(self.0.clone())
    }
}

#[tokio::test]
async fn test_the_page_lists_every_setting_with_stored_value_or_default() {
    let stored = su_configuration::Model {
        id: Uuid::now_v7(),
        category: "security".into(),
        key: "security.login.lock_minutes".into(),
        value: "30".into(),
        description: Some("how long".into()),
        change_by: None,
        row_version: 4,
    };
    let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let exec = Supg006CommandExecutor::new(Arc::new(Rows(vec![stored])), db);
    let items = exec.list_config().await.unwrap();

    assert_eq!(items.len(), CONFIG_SETTINGS.len());
    let lock = items.iter().find(|i| i.key == "security.login.lock_minutes").unwrap();
    assert_eq!((lock.value.as_str(), lock.row_version, lock.default_value.as_str()), ("30", 4, "15"));
    assert_eq!((lock.min, lock.max), (Some(1), Some(10080)));
    let missing = items.iter().find(|i| i.key == "security.login.max_failed_attempts").unwrap();
    assert_eq!((missing.value.as_str(), missing.row_version), ("5", 0), "no row: the default, version 0");
    assert!(items.iter().any(|i| i.group == ConfigGroup::MailTemplate));
    assert!(items.iter().all(|i| i.key != "mail.smtp_url" && i.key != "mail.from"));
    let _ = DbSupg006Queries::new; // the Sea-ORM implementation exists beside the in-memory one
}
