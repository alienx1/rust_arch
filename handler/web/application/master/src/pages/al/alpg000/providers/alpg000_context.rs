//! ALPG000 list context: filters, server paging and the audit-service status.
//!
//! Any filter change returns to page 1. The time window is turned into `from` at request time.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use web_package::core::i18n::t_or;

use crate::pages::al::alpg000::models::{
    AuditFilterField, AuditFilters, AuditLogPageDto, AuditLogSummaryDto, ProgramOptionDto,
};
use crate::pages::al::alpg000::services::{Alpg000Error, Alpg000Service};
use crate::routes::alpg000_log_path;

/// Rows per page of the log list
pub const PAGE_SIZE: u64 = 25;

#[derive(Clone, Copy)]
pub struct Alpg000Context {
    pub page: ReadSignal<AuditLogPageDto>,
    pub filters: ReadSignal<AuditFilters>,
    /// Catalogue programs offered by the program filter
    pub programs: ReadSignal<Vec<ProgramOptionDto>>,
    pub is_loading: ReadSignal<bool>,
    /// Quickwit is down (status call said so, or the list answered 503)
    pub unavailable: ReadSignal<bool>,
    /// Server message of a 503, when there was one
    pub unavailable_message: ReadSignal<Option<String>>,
    pub error: ReadSignal<Option<String>>,
    /// Sets one filter and returns to page 1
    pub set_filter: Callback<(AuditFilterField, String)>,
    pub go_to_page: Callback<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub refresh: Callback<()>,
    pub open_detail: Callback<AuditLogSummaryDto>,
    /// Outcome of the last "export CSV" press: `(ok, message)`
    pub export_notice: ReadSignal<Option<(bool, String)>>,
    /// Queues a CSV of everything the current filters match (built in the background)
    pub request_export: Callback<()>,
}

pub fn provide_alpg000_context() -> Alpg000Context {
    let service = Alpg000Service::new();

    let (page, set_page_data) = signal(AuditLogPageDto::default());
    let (filters, set_filters) = signal(AuditFilters::default());
    let (programs, set_programs) = signal(Vec::<ProgramOptionDto>::new());
    let (is_loading, set_is_loading) = signal(true);
    let (unavailable, set_unavailable) = signal(false);
    let (unavailable_message, set_unavailable_message) = signal(Option::<String>::None);
    let (error, set_error) = signal(Option::<String>::None);
    let (page_number, set_page_number) = signal(1u64);
    let (page_size, set_page_size) = signal(PAGE_SIZE);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    // The program filter's options load once; a failure just leaves the dropdown at "All"
    {
        let s = service.clone();
        spawn_local(async move {
            if let Ok(list) = s.fetch_programs().await {
                set_programs.set(list);
            }
        });
    }

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let filters = filters.get();
        let page_no = page_number.get();
        let limit = page_size.get();

        set_is_loading.set(true);
        set_error.set(None);
        let s = service.clone();
        spawn_local(async move {
            // The status call only decides the banner; the list call has the last word
            let down_by_status = matches!(s.fetch_status().await, Ok(st) if !st.available);
            set_unavailable.set(down_by_status);
            set_unavailable_message.set(None);
            if down_by_status {
                set_page_data.set(AuditLogPageDto::default());
                set_is_loading.set(false);
                return;
            }
            match s.fetch_logs(&filters, page_no, limit).await {
                Ok(v) => set_page_data.set(v),
                Err(Alpg000Error::Unavailable(message)) => {
                    set_page_data.set(AuditLogPageDto::default());
                    set_unavailable.set(true);
                    set_unavailable_message.set(Some(message).filter(|m| !m.trim().is_empty()));
                }
                Err(e) => {
                    set_page_data.set(AuditLogPageDto::default());
                    set_error.set(Some(e.message()));
                }
            }
            set_is_loading.set(false);
        });
    });

    let set_filter = Callback::new(move |(field, value): (AuditFilterField, String)| {
        set_filters.update(|f| f.set(field, value));
        set_page_number.set(1);
    });
    let go_to_page = Callback::new(move |p: u64| set_page_number.set(p.max(1)));
    let set_limit = Callback::new(move |n: u64| {
        set_page_size.set(n.clamp(1, 200));
        set_page_number.set(1);
    });
    let refresh = Callback::new(move |()| set_refresh_trigger.update(|v| *v = v.wrapping_add(1)));

    let navigate = use_navigate();
    let open_detail = Callback::new(move |row: AuditLogSummaryDto| {
        navigate(&alpg000_log_path(&row.event_id), Default::default())
    });

    let (export_notice, set_export_notice) = signal(Option::<(bool, String)>::None);
    let request_export = {
        let service = Alpg000Service::new();
        Callback::new(move |()| {
            set_export_notice.set(Some((true, t_or("alpg000.export.sending", "กำลังส่งคำขอ..."))));
            let s = service.clone();
            let filters = filters.get_untracked();
            spawn_local(async move {
                let notice = match s.request_export(&filters).await {
                    Ok(_) => (
                        true,
                        t_or(
                            "alpg000.export.queued",
                            "รับคำขอแล้ว ระบบกำลังสร้างไฟล์ CSV และจะแจ้งเตือนเมื่อพร้อม (ดูได้ที่ รายงานของฉัน)",
                        ),
                    ),
                    Err(e) => (false, e.message()),
                };
                set_export_notice.set(Some(notice));
            });
        })
    };

    let ctx = Alpg000Context {
        page,
        filters,
        programs,
        is_loading,
        unavailable,
        unavailable_message,
        error,
        set_filter,
        go_to_page,
        set_limit,
        refresh,
        open_detail,
        export_notice,
        request_export,
    };
    provide_context(ctx);
    ctx
}

pub fn use_alpg000() -> Alpg000Context {
    use_context::<Alpg000Context>()
        .expect("Alpg000Context must be provided by provide_alpg000_context()")
}
