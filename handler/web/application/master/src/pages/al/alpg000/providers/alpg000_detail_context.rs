//! ALPG000 detail context: loads one audit event and returns to the list.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::al::alpg000::models::AuditLogDto;
use crate::pages::al::alpg000::services::{Alpg000Error, Alpg000Service};
use crate::routes::ALPG000;

#[derive(Clone, Copy)]
pub struct Alpg000DetailContext {
    pub event: ReadSignal<Option<AuditLogDto>>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub not_found: ReadSignal<bool>,
    pub back: Callback<()>,
}

pub fn provide_alpg000_detail_context(id: String) -> Alpg000DetailContext {
    let service = Alpg000Service::new();
    let (event, set_event) = signal(Option::<AuditLogDto>::None);
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (not_found, set_not_found) = signal(false);

    spawn_local(async move {
        match service.fetch_log(&id).await {
            Ok(v) => set_event.set(Some(v)),
            Err(Alpg000Error::NotFound) => set_not_found.set(true),
            Err(e) => set_error.set(Some(e.message())),
        }
        set_is_loading.set(false);
    });

    let navigate = use_navigate();
    let back = Callback::new(move |()| navigate(ALPG000, Default::default()));

    let ctx = Alpg000DetailContext { event, is_loading, error, not_found, back };
    provide_context(ctx);
    ctx
}
