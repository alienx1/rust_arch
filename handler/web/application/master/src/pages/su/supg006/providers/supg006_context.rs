//! SUPG006 context: loads the settings, keeps the edits locally and saves only what changed.

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;
use web_package::core::i18n::t_or;

use crate::audit::track_action;
use crate::pages::su::supg006::models::{ConfigChangeInput, ConfigItemDto, SaveConfigInput};
use crate::pages::su::supg006::services::Supg006Service;

/// State of the settings page (`/su/supg006`)
#[derive(Clone, Copy)]
pub struct Supg006Context {
    pub items: ReadSignal<Vec<ConfigItemDto>>,
    /// Values typed since the last load / save: key -> value
    pub edits: ReadSignal<HashMap<String, String>>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub notice: ReadSignal<Option<String>>,
    pub set_value: Callback<(String, String)>,
    pub discard: Callback<()>,
    pub save: Callback<()>,
}

impl Supg006Context {
    /// What the field shows: the edited value, else the stored one
    pub fn value_of(&self, key: &str) -> String {
        self.edits
            .with(|e| e.get(key).cloned())
            .or_else(|| self.items.with(|items| items.iter().find(|i| i.key == key).map(|i| i.value.clone())))
            .unwrap_or_default()
    }

    /// The stored item of a key
    pub fn item(&self, key: &str) -> Option<ConfigItemDto> {
        self.items.with(|items| items.iter().find(|i| i.key == key).cloned())
    }

    /// Settings whose edited value differs from the stored one
    pub fn changed_count(&self) -> usize {
        changes(&self.items.get(), &self.edits.get()).len()
    }
}

fn changes(items: &[ConfigItemDto], edits: &HashMap<String, String>) -> Vec<ConfigChangeInput> {
    items
        .iter()
        .filter_map(|i| {
            let value = edits.get(&i.key)?;
            (value != &i.value).then(|| ConfigChangeInput { key: i.key.clone(), value: value.clone(), row_version: i.row_version })
        })
        .collect()
}

/// Provides Supg006Context into the reactive tree
pub fn provide_supg006_context() -> Supg006Context {
    let service = Supg006Service::new();
    let (items, set_items) = signal(Vec::<ConfigItemDto>::new());
    let (edits, set_edits) = signal(HashMap::<String, String>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (notice, set_notice) = signal(Option::<String>::None);

    {
        let s = service.clone();
        spawn_local(async move {
            match s.fetch_config().await {
                Ok(v) => set_items.set(v),
                Err(e) => set_error.set(Some(e)),
            }
            set_is_loading.set(false);
        });
    }

    let set_value = Callback::new(move |(key, value): (String, String)| {
        set_notice.set(None);
        set_edits.update(|e| {
            e.insert(key, value);
        });
    });
    let discard = Callback::new(move |()| {
        set_notice.set(None);
        set_error.set(None);
        set_edits.set(HashMap::new());
    });
    let save = Callback::new(move |()| {
        let list = changes(&items.get_untracked(), &edits.get_untracked());
        if list.is_empty() {
            set_notice.set(Some(t_or("supg006.nothing_changed", "ไม่มีรายการที่เปลี่ยนแปลง")));
            return;
        }
        let s = service.clone();
        set_is_saving.set(true);
        set_error.set(None);
        set_notice.set(None);
        spawn_local(async move {
            match s.save_config(SaveConfigInput { changes: list }).await {
                Ok(saved) => {
                    track_action("SUPG006.save", "");
                    set_items.set(saved);
                    set_edits.set(HashMap::new());
                    set_notice.set(Some(t_or("supg006.saved", "บันทึกการตั้งค่าเรียบร้อย (มีผลภายในประมาณ 1 นาที)")));
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_is_saving.set(false);
        });
    });

    let ctx = Supg006Context { items, edits, is_loading, is_saving, error, notice, set_value, discard, save };
    provide_context(ctx);
    ctx
}

/// Hook to retrieve the current Supg006Context
pub fn use_supg006() -> Supg006Context {
    use_context::<Supg006Context>().expect("Supg006Context must be provided by provide_supg006_context()")
}
