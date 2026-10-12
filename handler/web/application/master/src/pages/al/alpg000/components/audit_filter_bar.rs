//! Filter bar of the audit list: date range (from / to day, today by default), kind and status chips, user / method / program /
//! path inputs. Text inputs apply on Enter or when they lose focus.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Button, ButtonSize, ButtonVariant, DateRangePicker, FilterChips, Select, SelectOption};

use crate::pages::al::alpg000::models::{
    AuditFilterField, KIND_API, KIND_DATA_CHANGE, KIND_EVENT_NOTIFICATION, KIND_EVENT_REPORT_REQUEST, KIND_UI,
};
use crate::pages::al::alpg000::providers::use_alpg000;

const INPUT_CLASS: &str = "h-8 w-44 rounded-md border border-input bg-background px-2 text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring";

/// Text filter applied on `change` (Enter / blur), not on every keystroke
#[component]
fn TextFilter(
    label: String,
    placeholder: String,
    field: AuditFilterField,
    value: Signal<String>,
) -> impl IntoView {
    let ctx = use_alpg000();
    view! {
        <label class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground whitespace-nowrap">{label}</span>
            <input
                type="text"
                class=INPUT_CLASS
                placeholder=placeholder
                prop:value=move || value.get()
                on:change=move |ev| ctx.set_filter.run((field, event_target_value(&ev)))
            />
        </label>
    }
}

#[component]
pub fn AuditFilterBar() -> impl IntoView {
    let ctx = use_alpg000();
    let chips = |field: AuditFilterField| Callback::new(move |v: String| ctx.set_filter.run((field, v)));

    view! {
        <div class="space-y-1">
            <div class="flex flex-wrap items-center gap-2">
                <span class="text-xs text-muted-foreground whitespace-nowrap">
                    {t_or("alpg000.filter.date_range", "ช่วงวันที่:")}
                </span>
                <DateRangePicker
                    from=Signal::derive(move || ctx.filters.get().date_from)
                    to=Signal::derive(move || ctx.filters.get().date_to)
                    aria_label=t_or("alpg000.filter.date_range", "ช่วงวันที่:")
                    on_change=Callback::new(move |(from, to): (String, String)| {
                        ctx.set_filter.run((AuditFilterField::DateFrom, from));
                        ctx.set_filter.run((AuditFilterField::DateTo, to));
                    })
                />
            </div>
            <FilterChips
                label=t_or("alpg000.filter.kind", "ประเภท:")
                options=vec![
                    ("ALL".to_string(), t_or("alpg000.filter.kind_all", "ทั้งหมด")),
                    (KIND_UI.to_string(), t_or("alpg000.filter.kind_ui", "หน้าจอ (เปิดหน้า / คลิก / ดำเนินการ)")),
                    (KIND_API.to_string(), t_or("alpg000.filter.kind_api", "API")),
                    (KIND_DATA_CHANGE.to_string(), t_or("alpg000.filter.kind_data_change", "แก้ไขข้อมูล (ก่อน / หลัง)")),
                    (KIND_EVENT_REPORT_REQUEST.to_string(), t_or("alpg000.filter.kind_event_report_request", "Event: คำขอรายงาน (Redpanda)")),
                    (KIND_EVENT_NOTIFICATION.to_string(), t_or("alpg000.filter.kind_event_notification", "Event: การแจ้งเตือน (Redpanda)")),
                ]
                value=Signal::derive(move || ctx.filters.get().kind)
                on_change=chips(AuditFilterField::Kind)
            />
            <FilterChips
                label=t_or("alpg000.filter.status", "สถานะ:")
                options=vec![
                    ("ALL".to_string(), t_or("alpg000.filter.status_all", "ทั้งหมด")),
                    ("2".to_string(), "2xx".to_string()),
                    ("3".to_string(), "3xx".to_string()),
                    ("4".to_string(), "4xx".to_string()),
                    ("5".to_string(), "5xx".to_string()),
                ]
                value=Signal::derive(move || ctx.filters.get().status_class)
                on_change=chips(AuditFilterField::StatusClass)
            />
            <div class="flex flex-wrap items-center gap-x-4 gap-y-2 pt-2">
                <TextFilter
                    label=t_or("alpg000.filter.user", "ผู้ใช้:")
                    placeholder=t_or("alpg000.filter.user_placeholder", "รหัส / ชื่อผู้ใช้ / อีเมล")
                    field=AuditFilterField::User
                    value=Signal::derive(move || ctx.filters.get().user)
                />
                <label class="flex items-center gap-2">
                    <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("alpg000.filter.method", "เมธอด:")}</span>
                    <div class="w-28">
                        <Select
                            value=Signal::derive(move || ctx.filters.get().method)
                            on_change=chips(AuditFilterField::Method)
                            class="bg-background text-xs h-8"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("alpg000.filter.method_all", "ทั้งหมด")}</SelectOption>
                            <SelectOption value="GET".to_string()>"GET"</SelectOption>
                            <SelectOption value="POST".to_string()>"POST"</SelectOption>
                            <SelectOption value="PUT".to_string()>"PUT"</SelectOption>
                            <SelectOption value="DELETE".to_string()>"DELETE"</SelectOption>
                        </Select>
                    </div>
                </label>
                <label class="flex items-center gap-2">
                    <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("alpg000.filter.program", "โปรแกรม:")}</span>
                    <div class="w-64">
                        <Select
                            value=Signal::derive(move || ctx.filters.get().program_code)
                            on_change=chips(AuditFilterField::ProgramCode)
                            class="bg-background text-xs h-8"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("alpg000.filter.program_all", "ทั้งหมด")}</SelectOption>
                            {move || ctx.programs.get().into_iter().map(|p| {
                                let label = p.display_name();
                                view! { <SelectOption value=p.code.clone()>{label}</SelectOption> }
                            }).collect_view()}
                        </Select>
                    </div>
                </label>
                <TextFilter
                    label=t_or("alpg000.filter.path", "เส้นทาง:")
                    placeholder=t_or("alpg000.filter.path_placeholder", "ส่วนหนึ่งของ path")
                    field=AuditFilterField::Path
                    value=Signal::derive(move || ctx.filters.get().path)
                />
                <Button
                    size=ButtonSize::Sm
                    variant=ButtonVariant::Outline
                    on_click=Callback::new(move |_| ctx.request_export.run(()))
                >
                    {t_or("alpg000.export_csv", "ส่งออก CSV")}
                </Button>
                {move || ctx.export_notice.get().map(|(ok, text)| view! {
                    <span class=if ok { "text-xs text-muted-foreground" } else { "text-xs text-destructive font-medium" }>{text}</span>
                })}
            </div>
        </div>
    }
}
