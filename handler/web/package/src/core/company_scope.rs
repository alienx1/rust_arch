//! The company (and branch) chosen in the sidebar switcher, kept in `localStorage` (`company_id`, `branch_id`) and sent with
//! every API call as `X-Company-Id` / `X-Branch-Id`. The server decides what it may be (a headquarters account may
//! pick any company of its organization, any other account is limited to its own branch whatever
//! is sent), so this is only a request.

const STORAGE_KEY: &str = "company_id";

/// The company picked by the user (`None` = all companies: the overview)
pub fn get_selected_company() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok().flatten()?;
    storage.get_item(STORAGE_KEY).ok().flatten().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Remembers the pick (`None` = the overview)
pub fn save_selected_company(company_id: Option<&str>) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        match company_id {
            Some(id) => {
                let _ = storage.set_item(STORAGE_KEY, id);
            }
            None => {
                let _ = storage.remove_item(STORAGE_KEY);
            }
        }
    }
}

const BRANCH_KEY: &str = "branch_id";

/// The branch picked by the user (`None` = all branches)
pub fn get_selected_branch() -> Option<String> {
    let storage = web_sys::window()?.local_storage().ok().flatten()?;
    storage.get_item(BRANCH_KEY).ok().flatten().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Remembers the pick (`None` = all branches)
pub fn save_selected_branch(branch_id: Option<&str>) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        match branch_id {
            Some(id) => {
                let _ = storage.set_item(BRANCH_KEY, id);
            }
            None => {
                let _ = storage.remove_item(BRANCH_KEY);
            }
        }
    }
}
