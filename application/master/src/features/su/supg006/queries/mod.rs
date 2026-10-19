//! SUPG006 queries

use async_trait::async_trait;

use crate::features::su::supg006::errors::Supg006Error;
use domain::entity::su_configuration;

pub mod db_supg006_queries;

pub use db_supg006_queries::DbSupg006Queries;

#[async_trait]
pub trait Supg006Queries: Send + Sync {
    /// The `su_configuration` rows of the keys this program edits (rows that do not exist are absent)
    async fn list_rows(&self) -> Result<Vec<su_configuration::Model>, Supg006Error>;
}
