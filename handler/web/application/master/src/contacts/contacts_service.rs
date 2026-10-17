//! HTTP calls for the contacts of a record: `GET` / `PUT {program API}{record path}/contacts`

use leptos::prelude::*;
use leptos::task::spawn_local;
use web_package::core::i18n::t_or;
use web_package::ui::component::ContactItem;

use crate::api_client::ApiClient;

fn what() -> String {
    t_or("common.what.contacts", "ช่องทางติดต่อ")
}

/// Contacts of the record at `path` (e.g. `/organizations/{id}`) of the program API `base`
async fn fetch_contacts(base: &'static str, path: &str) -> Result<Vec<ContactItem>, String> {
    ApiClient::new(base).get(&format!("{path}/contacts"), &what()).await
}

/// Reads the contacts into `rows` (a failure leaves them empty; the form shows no error for it)
pub fn load_contacts(base: &'static str, path: String, rows: RwSignal<Vec<ContactItem>>) {
    spawn_local(async move {
        if let Ok(list) = fetch_contacts(base, &path).await {
            rows.set(list);
        }
    });
}

/// Replaces the contacts of the record with `rows`; rows with no value are dropped first
pub async fn save_contacts(base: &'static str, path: &str, rows: &[ContactItem]) -> Result<(), String> {
    let list: Vec<ContactItem> = rows
        .iter()
        .filter(|c| !c.value.trim().is_empty())
        .map(|c| ContactItem { value: c.value.trim().to_string(), ..c.clone() })
        .collect();
    ApiClient::new(base).put_cbor::<_, Vec<ContactItem>>(&format!("{path}/contacts"), &list, &what()).await.map(|_| ())
}
