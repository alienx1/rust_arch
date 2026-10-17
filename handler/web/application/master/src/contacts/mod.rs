//! Contacts of a record (phone, fax, e-mail, Line, tax id, ID card): the calls every record form
//! makes to read and replace them. The editor itself is `web_package::ui::component::ContactsEditor`.

mod contacts_service;

pub use contacts_service::{load_contacts, save_contacts};
