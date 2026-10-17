//! Contact commands: read and replace the contacts of a record

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::db::contacts::errors::ContactError;
use crate::features::db::contacts::models::{ContactDto, ContactInput, ContactOwner};

mod contact_executor;
mod list_contacts;
mod replace_contacts;

pub use contact_executor::ContactCommandExecutor;

#[async_trait]
pub trait ContactCommands: Send + Sync {
    /// Contacts of the record, default of each kind first; empty when it has none. `NotFound`
    /// when the record does not exist.
    async fn list_contacts(&self, owner: ContactOwner, id: Uuid) -> Result<Vec<ContactDto>, ContactError>;

    /// Replaces all contacts of the record with `contacts` (validated per kind; see
    /// `normalize_contacts`) and returns them as stored. The record's row version is left alone.
    async fn replace_contacts(
        &self,
        owner: ContactOwner,
        id: Uuid,
        contacts: Vec<ContactInput>,
        actor_id: Option<Uuid>,
    ) -> Result<Vec<ContactDto>, ContactError>;
}
