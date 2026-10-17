//! Account security: the sign-in lock, password mails and the forced password change.
//! Run: `cargo test -p application-master --test account_security_test`

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use application_core::traits::{
    IdentityError, IdentityProvisioningPort, InMemoryCache, MailError, MailMessage, MailService, NewIdentity,
    RecoveryLink, SessionRevocationPort,
};
use application_master::features::su::account_security::{
    AccountSecurityCommands, AccountSecurityError, AccountSecurityExecutor, AccountSecurityQueries, LoginState,
    PasswordMailKind, SecuritySettings,
};
use application_master::features::su::account_security::queries::AccountRef;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Default)]
struct Rows {
    accounts: Vec<AccountRef>,
    config: HashMap<String, String>,
}

#[derive(Default)]
struct MemoryQueries(Mutex<Rows>);

#[async_trait]
impl AccountSecurityQueries for MemoryQueries {
    async fn find_account(&self, id: Uuid) -> Result<Option<AccountRef>, AccountSecurityError> {
        Ok(self.0.lock().unwrap().accounts.iter().find(|a| a.id == id).cloned())
    }
    async fn find_account_by_identifier(&self, identifier: &str) -> Result<Option<AccountRef>, AccountSecurityError> {
        let name = identifier.split('@').next().unwrap().to_lowercase();
        Ok(self.0.lock().unwrap().accounts.iter().find(|a| a.username == name).cloned())
    }
    async fn set_must_change_password(&self, id: Uuid, value: bool) -> Result<bool, AccountSecurityError> {
        let mut rows = self.0.lock().unwrap();
        match rows.accounts.iter_mut().find(|a| a.id == id) {
            Some(a) => {
                a.must_change_password = value;
                Ok(true)
            }
            None => Ok(false),
        }
    }
    async fn security_configuration(&self) -> Result<HashMap<String, String>, AccountSecurityError> {
        Ok(self.0.lock().unwrap().config.clone())
    }
}

#[derive(Default)]
struct Identity {
    emails: HashMap<String, String>,
}

#[async_trait]
impl IdentityProvisioningPort for Identity {
    async fn create_identity(&self, _i: NewIdentity) -> Result<String, IdentityError> {
        unreachable!()
    }
    async fn delete_identity(&self, _id: &str) -> Result<(), IdentityError> {
        unreachable!()
    }
    async fn create_recovery_link(&self, identifier: &str) -> Result<RecoveryLink, IdentityError> {
        Ok(RecoveryLink { url: format!("http://localhost:3000/recovery?flow=abc&user={identifier}"), expires_at: None })
    }
    async fn find_identity_id(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        Ok(self.emails.get(identifier).map(|_| "identity".to_string()))
    }
    async fn find_email(&self, identifier: &str) -> Result<Option<String>, IdentityError> {
        Ok(self.emails.get(identifier).cloned())
    }
}

#[derive(Default)]
struct Outbox {
    sent: Mutex<Vec<MailMessage>>,
    fail: Mutex<bool>,
}

#[async_trait]
impl MailService for Outbox {
    async fn send(&self, message: MailMessage) -> Result<(), MailError> {
        if *self.fail.lock().unwrap() {
            return Err(MailError::NotConfigured);
        }
        self.sent.lock().unwrap().push(message);
        Ok(())
    }
}

#[derive(Default)]
struct Revocations(Mutex<Vec<String>>);

#[async_trait]
impl SessionRevocationPort for Revocations {
    async fn revoke_session(&self, _s: &str) -> Result<bool, String> {
        Ok(true)
    }
    async fn revoke_user_sessions(&self, user: &str) -> Result<bool, String> {
        self.0.lock().unwrap().push(user.to_string());
        Ok(true)
    }
    async fn set_user_active(&self, _u: &str, _a: bool) -> Result<bool, String> {
        Ok(true)
    }
    async fn list_user_sessions(&self, _u: &str) -> Result<Vec<application_core::traits::IdentitySessionInfo>, String> {
        Ok(vec![])
    }
}

struct Rig {
    exec: AccountSecurityExecutor,
    queries: Arc<MemoryQueries>,
    outbox: Arc<Outbox>,
    revoked: Arc<Revocations>,
    somchai: Uuid,
}

fn rig() -> Rig {
    let somchai = Uuid::now_v7();
    let queries = Arc::new(MemoryQueries::default());
    queries.0.lock().unwrap().accounts = vec![
        AccountRef { id: somchai, username: "somchai".into(), active: true, must_change_password: false },
        AccountRef { id: Uuid::now_v7(), username: "off".into(), active: false, must_change_password: false },
    ];
    let identity = Arc::new(Identity {
        emails: HashMap::from([("somchai".to_string(), "somchai@arch.local".to_string())]),
    });
    let (outbox, revoked) = (Arc::new(Outbox::default()), Arc::new(Revocations::default()));
    let exec = AccountSecurityExecutor::new(queries.clone(), Arc::new(InMemoryCache::new()))
        .with_identity(identity)
        .with_mail(outbox.clone())
        .with_session_revocation(revoked.clone());
    Rig { exec, queries, outbox, revoked, somchai }
}

fn set_config(r: &Rig, key: &str, value: &str) {
    r.queries.0.lock().unwrap().config.insert(key.to_string(), value.to_string());
}

// ---------- the sign-in lock

#[tokio::test]
async fn test_five_failures_lock_the_account_and_a_success_clears_the_count() {
    let r = rig();
    for _ in 0..4 {
        assert_eq!(r.exec.record_login_failure("somchai").await.unwrap(), LoginState::Open);
    }
    assert_eq!(r.exec.login_state("somchai").await.unwrap(), LoginState::Open);
    let locked = r.exec.record_login_failure("somchai").await.unwrap();
    assert_eq!(locked, LoginState::Locked { retry_after_secs: 15 * 60 }, "the 5th failure locks for the configured 15 minutes");
    assert!(matches!(r.exec.login_state("somchai").await.unwrap(), LoginState::Locked { .. }));

    // A password change also lifts the lock
    r.exec.mark_password_changed("somchai").await.unwrap();
    assert_eq!(r.exec.login_state("somchai").await.unwrap(), LoginState::Open);

    // Failures that did not reach the limit are forgotten after a successful sign-in
    for _ in 0..4 {
        r.exec.record_login_failure("somchai").await.unwrap();
    }
    r.exec.record_login_success("somchai").await.unwrap();
    for _ in 0..4 {
        assert_eq!(r.exec.record_login_failure("somchai").await.unwrap(), LoginState::Open, "the count started again");
    }
}

#[tokio::test]
async fn test_the_numbers_come_from_su_configuration() {
    let r = rig();
    set_config(&r, "security.login.max_failed_attempts", "2");
    set_config(&r, "security.login.lock_minutes", "30");
    assert_eq!(r.exec.record_login_failure("somchai").await.unwrap(), LoginState::Open);
    assert_eq!(r.exec.record_login_failure("somchai").await.unwrap(), LoginState::Locked { retry_after_secs: 30 * 60 });

    // 0 switches the lock off
    let off = rig();
    set_config(&off, "security.login.max_failed_attempts", "0");
    for _ in 0..20 {
        assert_eq!(off.exec.record_login_failure("somchai").await.unwrap(), LoginState::Open);
    }
}

#[tokio::test]
async fn test_settings_fall_back_to_safe_defaults() {
    let mut rows = HashMap::new();
    assert_eq!(SecuritySettings::from_rows(&rows), SecuritySettings::default());
    rows.insert("security.login.max_failed_attempts".to_string(), "not a number".to_string());
    rows.insert("security.login.lock_minutes".to_string(), "0".to_string());
    let s = SecuritySettings::from_rows(&rows);
    assert_eq!(s.max_failed_attempts, 5, "unreadable: default");
    assert_eq!(s.lock_minutes, 1, "a zero-minute lock is raised to one, not switched off");
}

#[tokio::test]
async fn test_an_account_is_one_name_in_any_spelling_and_unknown_names_are_counted_too() {
    let r = rig();
    for who in ["somchai", "SomChai", "somchai@arch.local", " somchai ", "SOMCHAI@x.io"] {
        r.exec.record_login_failure(who).await.unwrap();
    }
    assert!(matches!(r.exec.login_state("somchai").await.unwrap(), LoginState::Locked { .. }), "five spellings, one account");

    // A name that does not exist locks the same way: the answer never says whether it exists
    for _ in 0..5 {
        r.exec.record_login_failure("nobody").await.unwrap();
    }
    assert!(matches!(r.exec.login_state("nobody").await.unwrap(), LoginState::Locked { .. }));
    // and somebody else is untouched
    assert_eq!(r.exec.login_state("other").await.unwrap(), LoginState::Open);
}

// ---------- password mails

#[tokio::test]
async fn test_a_password_mail_carries_the_link_and_says_where_it_went() {
    let r = rig();
    let result = r.exec.send_password_mail(r.somchai, PasswordMailKind::Setup).await.unwrap();
    assert!(result.sent && result.recovery_link.is_none(), "the link is not handed out when the mail went");
    assert_eq!(result.sent_to.as_deref(), Some("s***@arch.local"), "the address is shown masked");

    let sent = r.outbox.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "somchai@arch.local");
    assert!(sent[0].text.contains("recovery?flow=abc") && sent[0].text.contains("somchai"));
    assert!(sent[0].subject.contains("รหัสผ่าน") && sent[0].subject.contains("password"), "Thai and English");
    assert!(sent[0].html.as_ref().unwrap().contains("href=\"http://localhost:3000/recovery?flow=abc"));
}

#[tokio::test]
async fn test_when_no_mail_can_be_sent_the_administrator_gets_the_link_instead() {
    let r = rig();
    *r.outbox.fail.lock().unwrap() = true;
    let result = r.exec.send_password_mail(r.somchai, PasswordMailKind::Reset).await.unwrap();
    assert!(!result.sent);
    assert!(result.mail_error.unwrap().contains("smtp_url"));
    assert!(result.recovery_link.unwrap().contains("recovery?flow=abc"));
}

#[tokio::test]
async fn test_no_mail_for_an_unknown_switched_off_or_address_less_account() {
    let r = rig();
    assert!(matches!(r.exec.send_password_mail(Uuid::now_v7(), PasswordMailKind::Reset).await, Err(AccountSecurityError::NotFound(_))));
    let off = r.queries.0.lock().unwrap().accounts[1].id;
    assert!(matches!(r.exec.send_password_mail(off, PasswordMailKind::Reset).await, Err(AccountSecurityError::Validation(_))));
    r.queries.0.lock().unwrap().accounts.push(AccountRef { id: Uuid::now_v7(), username: "noemail".into(), active: true, must_change_password: false });
    let id = r.queries.0.lock().unwrap().accounts.last().unwrap().id;
    assert!(matches!(r.exec.send_password_mail(id, PasswordMailKind::Reset).await, Err(AccountSecurityError::NotFound(_))));
    assert!(r.outbox.sent.lock().unwrap().is_empty());
}

// ---------- forced password change

#[tokio::test]
async fn test_forcing_a_change_blocks_the_account_ends_its_sessions_and_mails_the_link() {
    let r = rig();
    assert!(!r.exec.password_change_required("somchai").await.unwrap());
    let result = r.exec.force_password_change(r.somchai).await.unwrap();
    assert!(result.sent);
    assert!(r.exec.password_change_required("somchai@arch.local").await.unwrap(), "now it cannot sign in");
    assert_eq!(r.revoked.0.lock().unwrap().as_slice(), ["somchai"], "whoever was signed in is signed out");
    assert!(r.outbox.sent.lock().unwrap()[0].subject.contains("เปลี่ยนรหัสผ่าน"));

    // The new password clears it
    r.exec.mark_password_changed("somchai").await.unwrap();
    assert!(!r.exec.password_change_required("somchai").await.unwrap());
}

#[tokio::test]
async fn test_trying_to_sign_in_while_a_change_is_required_mails_the_link_again_but_not_in_a_loop() {
    let r = rig();
    r.exec.force_password_change(r.somchai).await.unwrap();
    r.outbox.sent.lock().unwrap().clear();

    r.exec.notify_password_required("somchai").await;
    assert_eq!(r.outbox.sent.lock().unwrap().len(), 1, "the first attempt mails the link");
    for _ in 0..5 {
        r.exec.notify_password_required("somchai").await;
    }
    assert_eq!(r.outbox.sent.lock().unwrap().len(), 1, "further attempts within the interval do not mail again");

    // Somebody who does not have to change anything gets no mail, and neither does an unknown name
    let other = rig();
    other.exec.notify_password_required("somchai").await;
    other.exec.notify_password_required("nobody").await;
    assert!(other.outbox.sent.lock().unwrap().is_empty());
}

#[test]
fn test_mail_templates_from_configuration_replace_the_built_in_text() {
    use application_master::features::su::account_security::{PasswordMailKind, password_mail_message};
    use std::collections::HashMap;

    let url = "http://k/recovery?flow=1&token=a<b";
    let none = password_mail_message(PasswordMailKind::Reset, "a@b.c", "somchai", url, 24, &HashMap::new());
    assert!(none.text.contains("somchai") && none.text.contains(url), "the built-in text is used without templates");

    let templates: HashMap<String, String> = [
        ("mail.template.app_name", "Arch <ERP>"),
        ("mail.template.reset.subject", "Hello {{username}} @ {{app_name}}"),
        ("mail.template.reset.html", "<a href=\"{{link}}\">{{username}}</a> {{hours}}h {{app_name}}"),
        ("mail.template.reset.text", "{{link}} for {{username}} ({{hours}}h)"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let m = password_mail_message(PasswordMailKind::Reset, "a@b.c", "somchai", url, 24, &templates);
    assert_eq!(m.subject, "Hello somchai @ Arch <ERP>", "subject and text are plain");
    assert_eq!(m.text, format!("{url} for somchai (24h)"));
    assert_eq!(
        m.html.unwrap(),
        "<a href=\"http://k/recovery?flow=1&amp;token=a&lt;b\">somchai</a> 24h Arch &lt;ERP&gt;",
        "values are escaped in the HTML"
    );

    // A kind without its own rows keeps the built-in text
    let other = password_mail_message(PasswordMailKind::Setup, "a@b.c", "somchai", url, 24, &templates);
    assert!(other.subject.contains("Set your password"));
}
