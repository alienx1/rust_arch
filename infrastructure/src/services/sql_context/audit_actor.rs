//! Who is behind a database write: the caller of the current API request, handed to Postgres.
//!
//! The audit middleware wraps a write request in [`with_actor`]; every connection the request
//! takes from the pool then carries the actor in three session settings (`app.actor_id`,
//! `app.actor_name`, `app.request_id`) which the `al_capture_change` trigger copies into
//! `al_data_change`. Connections taken outside such a request get the settings cleared, so a
//! pooled connection never carries a previous caller into a background write (which is then not
//! captured at all: seeds, migrations and workers are not user actions).

use sea_orm::ConnectOptions;
use sea_orm::sqlx::Executor;
use std::future::Future;

/// The caller of one request
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditActor {
    /// Kratos identity id
    pub user_id: String,
    pub username: String,
    /// `event_id` of the `kind=api` audit event of the same request
    pub request_id: String,
}

tokio::task_local! {
    static ACTOR: AuditActor;
}

/// Runs `fut` with `actor` as the caller of every database write it makes (on this task)
pub async fn with_actor<F: Future>(actor: AuditActor, fut: F) -> F::Output {
    ACTOR.scope(actor, fut).await
}

/// The actor of the current task, if any
pub fn current_actor() -> Option<AuditActor> {
    ACTOR.try_with(Clone::clone).ok()
}

/// Makes every pool acquire publish the current actor to the session (see module docs)
pub fn track_actor(opt: &mut ConnectOptions) {
    opt.map_sqlx_postgres_pool_opts(|pool| {
        pool.before_acquire(|conn, _meta| {
            Box::pin(async move {
                let (id, name, request) = current_actor()
                    .map(|a| (a.user_id, a.username, a.request_id))
                    .unwrap_or_default();
                conn.execute(
                    sea_orm::sqlx::query(
                        "SELECT set_config('app.actor_id', $1, false), \
                                set_config('app.actor_name', $2, false), \
                                set_config('app.request_id', $3, false)",
                    )
                    .bind(id)
                    .bind(name)
                    .bind(request),
                )
                .await?;
                Ok(true)
            })
        })
    });
}
