//! Who asked for the mail being sent (the administrator who pressed the button).
//!
//! The API runs the command inside [`scope`]; the account security feature copies [`current`] into
//! the queued message, so the mail delivery can tell that person when the mail could not be sent.
//! Outside a scope (the holder signing in again, background jobs, tests) there is nobody to tell.

use std::future::Future;

tokio::task_local! {
    static REQUESTER: String;
}

/// Runs `future` with `username` as the requester of any mail it sends
pub async fn scope<F: Future>(username: String, future: F) -> F::Output {
    REQUESTER.scope(username, future).await
}

/// The requester of the running request, if any
pub fn current() -> Option<String> {
    REQUESTER.try_with(|u| u.clone()).ok()
}
