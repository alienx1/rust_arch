//! Filter bar of the process list: request date range and status chips (with counters).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{DateRangePicker, FilterChips};

use crate::pages::al::alpg001::models::{ProcessFilterField, STATUS_FAILED, STATUS_QUEUED, STATUS_RUNNING, STATUS_SUCCEEDED};
use crate::pages::al::alpg001::providers::use_alpg001;

#[component]
pub fn ProcessFilterBar() -> impl IntoView {
    let ctx = use_alpg001();
    // Chip text with its counter, e.g. "ไม่สำเร็จ (3)"
    let counted = move |key: &'static str, text: String| {
        let c = ctx.page.get().counts;
        let n = match key {
            STATUS_QUEUED => c.queued,
            STATUS_RUNNING => c.running,
            STATUS_SUCCEEDED => c.succeeded,
            STATUS_FAILED => c.failed,
            _ => c.queued + c.running + c.succeeded + c.failed,
        };
        format!("{text} ({n})")
    };

    view! {
        <div class="space-y-1">
            <div class="flex flex-wrap items-center gap-2">
                <span class="text-xs text-muted-foreground whitespace-nowrap">
                    {t_or("alpg001.filter.date_range", "วันที่ขอ:")}
                </span>
                <DateRangePicker
                    from=Signal::derive(move || ctx.filters.get().date_from)
                    to=Signal::derive(move || ctx.filters.get().date_to)
                    aria_label=t_or("alpg001.filter.date_range", "วันที่ขอ:")
                    on_change=Callback::new(move |(from, to): (String, String)| {
                        ctx.set_filter.run((ProcessFilterField::DateFrom, from));
                        ctx.set_filter.run((ProcessFilterField::DateTo, to));
                    })
                />
            </div>
            {move || view! {
                <FilterChips
                    label=t_or("alpg001.filter.status", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), counted("ALL", t_or("alpg001.filter.status_all", "ทั้งหมด"))),
                        (STATUS_FAILED.to_string(), counted(STATUS_FAILED, t_or("alpg001.status.failed", "ไม่สำเร็จ"))),
                        (STATUS_RUNNING.to_string(), counted(STATUS_RUNNING, t_or("alpg001.status.running", "กำลังทำงาน"))),
                        (STATUS_QUEUED.to_string(), counted(STATUS_QUEUED, t_or("alpg001.status.queued", "รอคิว"))),
                        (STATUS_SUCCEEDED.to_string(), counted(STATUS_SUCCEEDED, t_or("alpg001.status.succeeded", "สำเร็จ"))),
                    ]
                    value=Signal::derive(move || ctx.filters.get().status)
                    on_change=Callback::new(move |v: String| ctx.set_filter.run((ProcessFilterField::Status, v)))
                />
            }}
        </div>
    }
}
