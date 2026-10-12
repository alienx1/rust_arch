use super::client::KratosClient;
use super::types::{KratosIdentity, KratosSession};
use leptos::prelude::*;

/// Reactive authentication context provided to Leptos component tree.
#[derive(Clone, Copy)]
pub struct AuthContext {
    pub session: ReadSignal<Option<KratosSession>>,
    pub user: Signal<Option<KratosIdentity>>,
    pub is_authenticated: Signal<bool>,
    pub is_loading: ReadSignal<bool>,
    set_session: WriteSignal<Option<KratosSession>>,
    set_is_loading: WriteSignal<bool>,
    client: &'static KratosClient,
}

impl AuthContext {
    /// Reference to the underlying Kratos API client.
    pub fn client(&self) -> &'static KratosClient {
        self.client
    }

    /// Refresh session from Kratos (`/sessions/whoami`).
    pub fn check_session(&self) {
        let client = self.client;
        let set_session = self.set_session;
        let set_is_loading = self.set_is_loading;

        set_is_loading.set(true);
        leptos::task::spawn_local(async move {
            let session = client.whoami().await.unwrap_or(None);
            set_session.set(session);
            set_is_loading.set(false);
        });
    }

    /// Re-verify the session in the background (heartbeat / focus).
    ///
    /// Unlike [`check_session`](Self::check_session) it does not toggle `is_loading`
    /// and only writes the signal when the session actually changed, so downstream
    /// effects do not re-run on every poll.
    pub fn check_session_silent(&self) {
        let client = self.client;
        let session = self.session;
        let set_session = self.set_session;

        leptos::task::spawn_local(async move {
            let fresh = client.whoami().await.unwrap_or(None);
            if session.get_untracked() != fresh {
                set_session.set(fresh);
            }
        });
    }

    /// Perform logout and clear local session state.
    pub fn logout(&self) {
        let client = self.client;
        let set_session = self.set_session;

        leptos::task::spawn_local(async move {
            let _ = client.logout().await;
            set_session.set(None);
        });
    }

    /// Set session explicitly upon successful login.
    pub fn set_active_session(&self, session: KratosSession) {
        self.set_session.set(Some(session));
    }

    /// List all active sessions for the current authenticated user.
    pub async fn list_my_sessions(&self) -> Result<Vec<KratosSession>, crate::core::errors::AuthError> {
        self.client.list_my_sessions().await
    }

    /// Revoke all other active sessions except the current one.
    pub async fn disable_my_other_sessions(&self) -> Result<(), crate::core::errors::AuthError> {
        self.client.disable_my_other_sessions().await
    }

    /// Revoke a specific session by ID.
    pub async fn disable_my_session(&self, session_id: &str) -> Result<(), crate::core::errors::AuthError> {
        self.client.disable_my_session(session_id).await
    }
}

/// Initialize and provide the authentication context into Leptos root context.
///
/// Automatically performs an initial session check on mount.
pub fn provide_auth_context(kratos_url: impl Into<String>) -> AuthContext {
    let client = Box::leak(Box::new(KratosClient::new(kratos_url)));
    let (session, set_session) = signal::<Option<KratosSession>>(None);
    let (is_loading, set_is_loading) = signal::<bool>(true);

    let is_authenticated =
        Signal::derive(move || session.get().as_ref().map(|s| s.active).unwrap_or(false));

    let user = Signal::derive(move || session.get().and_then(|s| s.identity));

    let auth_ctx = AuthContext {
        session,
        user,
        is_authenticated,
        is_loading,
        set_session,
        set_is_loading,
        client,
    };

    provide_context(auth_ctx);

    // Initial session check
    auth_ctx.check_session();

    auth_ctx
}

/// Retrieve the active `AuthContext` from Leptos context hierarchy.
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}
