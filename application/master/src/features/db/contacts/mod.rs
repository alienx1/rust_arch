//! Contacts of organizations, companies, branches and employees: phone, fax, e-mail, Line, tax id,
//! ID card number. They all live in a contact group (`db_contact_group` + `db_contact`, the type is
//! a `contact_type` lookup) that the owner row points to with `contact_group_id`.
//!
//! The list is read and replaced as a whole through the owner's program API (organizations and
//! companies `DBPG001`, branches `DBPG002`, employees `DBPG003`).

mod contact_group_cleanup;
pub mod commands;
pub mod errors;
pub mod models;

pub use contact_group_cleanup::delete_contact_group;
pub use commands::{ContactCommandExecutor, ContactCommands};
pub use errors::ContactError;
pub use models::*;
