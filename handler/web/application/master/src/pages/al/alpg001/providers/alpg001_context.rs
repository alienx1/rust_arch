//! ALPG001 context: filters, server paging, the per-status counters and the reprocess action.
//! While a process is waiting or running the list refreshes itself every few seconds.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use web_package::core::i18n::t_or;

use crate::pages::al::alpg001::models::{
    ProcessDto, ProcessFilterField, ProcessFilters, ProcessPageDto,
};
use crate::pages::al::alpg001::services::Alpg001Service;

/// Rows per page
pub const PAGE_SIZE: u64 = 25;
/// How often the list reloads while something is waiting or running
const AUTO_REFRESH: Duration = Duration::from_secs(4);

#[derive(Clone, Copy)]
pub struct Alpg001Context {
    pub page: ReadSignal<ProcessPageDto>,
    pub filters: ReadSignal<ProcessFilters>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    /// Outcome of the last reprocess press: `(ok, message)`
    pub notice: ReadSignal<Option<(bool, String)>>,
    /// Id of the process whose reprocess request is being sent
    pub busy: ReadSignal<Option<String>>,
    /// Sets one filter and returns to page 1
    pub set_filter: Callback<(ProcessFilterField, String)>,
    pub go_to_page: Callback<u64>,
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub reprocess: Callback<ProcessDto>,
}

pub fn provide_alpg001_context() -> Alpg001Context {
    let service = Alpg001Service::new();

    let (page, set_page_data) = signal(ProcessPageDto::default());
    let (filters, set_filters) = signal(ProcessFilters::all());
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (notice, set_notice) = signal(Option::<(bool, String)>::None);
    let (busy, set_busy) = signal(Option::<String>::None);
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    // The page was left: a pending auto-refresh must not touch its signals any more
    let alive = Arc::new(AtomicBool::new(true));
    {
        let alive = alive.clone();
        on_cleanup(move || alive.store(false, Ordering::Relaxed));
    }

    Effect::new({
        let service = service.clone();
        move |_| {
            refresh_trigger.track();
            let filters = filters.get();
            let page_no = page_number.get();
            let limit = page_size.get();
            // Only the first load of a view shows the spinner; auto-refreshes stay quiet
            if page.with_untracked(|p| p.items.is_empty()) {
                set_is_loading.set(true);
            }
            let s = service.clone();
            let alive = alive.clone();
            spawn_local(async move {
                match s.fetch_processes(&filters, page_no, limit).await {
                    Ok(v) => {
                        let again = v.has_active();
                        set_page_data.set(v);
                        set_error.set(None);
                        if again {
                            set_timeout(
                                move || {
                                    if alive.load(Ordering::Relaxed) {
                                        set_refresh_trigger.update(|n| *n = n.wrapping_add(1));
                                    }
                                },
                                AUTO_REFRESH,
                            );
                        }
                    }
                    Err(e) => {
                        set_page_data.set(ProcessPageDto::default());
                        set_error.set(Some(e.message()));
                    }
                }
                set_is_loading.set(false);
            });
        }
    });

    let set_filter = Callback::new(move |(field, value): (ProcessFilterField, String)| {
        set_filters.update(|f| f.set(field, value));
        set_page_number.set(1);
    });
    let go_to_page = Callback::new(move |p: u64| set_page_number.set(p.max(1)));
    let set_limit = Callback::new(move |n: u64| {
        set_page_size.set(n.clamp(1, 200));
        set_page_number.set(1);
    });
    let refresh = Callback::new(move |()| set_refresh_trigger.update(|n| *n = n.wrapping_add(1)));

    let reprocess = Callback::new(move |process: ProcessDto| {
        let ask = t_or("alpg001.confirm.reprocess", "ประมวลผลรายการนี้ใหม่ตามคำขอเดิม?");
        let confirmed = web_sys::window().and_then(|w| w.confirm_with_message(&ask).ok()).unwrap_or(false);
        if !confirmed {
            return;
        }
        set_busy.set(Some(process.id.clone()));
        set_notice.set(None);
        let s = service.clone();
        spawn_local(async move {
            match s.reprocess(&process.id).await {
                Ok(()) => set_notice.set(Some((true, t_or("alpg001.notice.queued", "ส่งเข้าคิวประมวลผลใหม่แล้ว")))),
                Err(e) => set_notice.set(Some((false, e.message()))),
            }
            set_busy.set(None);
            set_refresh_trigger.update(|n| *n = n.wrapping_add(1));
        });
    });

    let ctx = Alpg001Context {
        page,
        filters,
        is_loading,
        error,
        notice,
        busy,
        set_filter,
        go_to_page,
        set_limit,
        refresh,
        reprocess,
    };
    provide_context(ctx);
    ctx
}

pub fn use_alpg001() -> Alpg001Context {
    expect_context::<Alpg001Context>()
}
