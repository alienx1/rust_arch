//! Audit log table (shared `DataTable` shell): time, kind, user, program/action, what, status,
//! duration, details. Search (full text), refresh, filters and server paging live in the shell.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, TableCell,
    TableRow,
};

use crate::pages::al::alpg000::components::audit_time_format::EventTime;
use crate::pages::al::alpg000::components::{AuditFilterBar, AuditStatusBanner};
use crate::pages::al::alpg000::models::{
    AuditLogSummaryDto, KIND_ACTION, KIND_API, KIND_CLICK, KIND_DATA_CHANGE, KIND_EVENT_NOTIFICATION, KIND_EVENT_REPORT_REQUEST,
    KIND_PAGE_VIEW,
};
use crate::pages::al::alpg000::providers::use_alpg000;

const SHIELD_ICON: &str = "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z";

/// Text of a kind, falling back to the raw value for kinds this build does not know
pub fn kind_label(kind: &str) -> String {
    match kind {
        KIND_PAGE_VIEW => t_or("alpg000.kind.page_view", "เปิดหน้า"),
        KIND_CLICK => t_or("alpg000.kind.click", "คลิก"),
        KIND_ACTION => t_or("alpg000.kind.action", "ดำเนินการ"),
        KIND_API => t_or("alpg000.kind.api", "API"),
        KIND_DATA_CHANGE => t_or("alpg000.kind.data_change", "แก้ไขข้อมูล"),
        KIND_EVENT_REPORT_REQUEST => t_or("alpg000.kind.event_report_request", "Event: คำขอรายงาน"),
        KIND_EVENT_NOTIFICATION => t_or("alpg000.kind.event_notification", "Event: แจ้งเตือน"),
        other if other.is_empty() => "-".to_string(),
        other => other.to_string(),
    }
}

pub fn kind_variant(kind: &str) -> BadgeVariant {
    match kind {
        KIND_PAGE_VIEW => BadgeVariant::Secondary,
        KIND_CLICK => BadgeVariant::Outline,
        KIND_ACTION => BadgeVariant::Success,
        KIND_API => BadgeVariant::Default,
        KIND_DATA_CHANGE => BadgeVariant::Warning,
        KIND_EVENT_REPORT_REQUEST | KIND_EVENT_NOTIFICATION => BadgeVariant::Secondary,
        _ => BadgeVariant::Outline,
    }
}

/// 2xx success, 3xx secondary, 4xx warning, 5xx destructive
pub fn status_variant(status: u16) -> BadgeVariant {
    match status / 100 {
        2 => BadgeVariant::Success,
        3 => BadgeVariant::Secondary,
        4 => BadgeVariant::Warning,
        5 => BadgeVariant::Destructive,
        _ => BadgeVariant::Outline,
    }
}

fn dash_if_blank(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[component]
pub fn AuditLogTable() -> impl IntoView {
    let ctx = use_alpg000();

    let columns = vec![
        DataColumn::new(t_or("alpg000.column.time", "เวลา")).class("w-40"),
        DataColumn::new(t_or("alpg000.column.kind", "ประเภท")).class("w-28"),
        DataColumn::new(t_or("alpg000.column.user", "ผู้ใช้")).class("min-w-[140px]"),
        DataColumn::new(t_or("alpg000.column.program", "โปรแกรม / การกระทำ")).class("min-w-[140px]"),
        DataColumn::new(t_or("alpg000.column.what", "รายละเอียด")).class("min-w-[240px]"),
        DataColumn::new(t_or("alpg000.column.status", "สถานะ")).class("w-20 text-center"),
        DataColumn::new(t_or("alpg000.column.duration", "ms")).class("w-20 text-right"),
        DataColumn::new(t_or("alpg000.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("alpg000.table.title", "บันทึกเหตุการณ์ (Audit Log)")
            description=t_or("alpg000.table.description", "การเข้าใช้งานของผู้ใช้ (หน้าที่เปิด ปุ่มที่กด กระบวนการที่ทำ) และบันทึกระบบของทุก API • เก็บใน Quickwit")
            icon_path=SHIELD_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("alpg000.state.loading", "กำลังโหลดบันทึกเหตุการณ์...")
            empty_title=t_or("alpg000.state.empty_title", "ไม่พบเหตุการณ์")
            empty_text=t_or("alpg000.state.empty_text", "ไม่พบรายการที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.filters.get().text)
            search_placeholder=t_or("alpg000.search.placeholder", "ค้นหาข้อความในบันทึก...")
            on_search=Callback::new(move |v: String| {
                ctx.set_filter.run((crate::pages::al::alpg000::models::AuditFilterField::Text, v))
            })
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            notice=move || view! { <AuditStatusBanner /> }
            filters=move || view! { <AuditFilterBar /> }
        >
            {move || ctx.page.get().items.into_iter().map(|row| view! { <AuditLogRow row=row /> }).collect_view()}
        </DataTable>
    }
}

#[component]
fn AuditLogRow(row: AuditLogSummaryDto) -> impl IntoView {
    let ctx = use_alpg000();
    let for_open = row.clone();
    let time = EventTime::parse(&row.timestamp);
    let time_title = time.as_ref().map(|t| t.utc.clone()).unwrap_or_else(|| row.timestamp.clone());

    let username = dash_if_blank(&row.username);
    let user_id = dash_if_blank(&row.user_id);
    let program = dash_if_blank(&row.program_code);
    let action = dash_if_blank(&row.action);
    let is_api = row.kind == KIND_API;

    let what = if is_api {
        let method = dash_if_blank(&row.method).unwrap_or_default();
        let path = dash_if_blank(&row.path).unwrap_or_default();
        view! {
            <div class="font-mono text-xs break-all">
                <span class="font-semibold">{method}</span>" "{path}
            </div>
        }
        .into_any()
    } else if row.kind == KIND_DATA_CHANGE {
        // "db_branch UPDATE, db_branch_label UPDATE ×2": what the one request changed
        let summary = dash_if_blank(&row.detail).unwrap_or_default();
        view! { <div class="font-mono text-xs break-all">{summary}</div> }.into_any()
    } else if row.kind == KIND_PAGE_VIEW {
        view! { <div class="font-mono text-xs break-all">{dash_if_blank(&row.page).unwrap_or_default()}</div> }.into_any()
    } else {
        let label = dash_if_blank(&row.label);
        let element = dash_if_blank(&row.element);
        let page = dash_if_blank(&row.page);
        let detail = dash_if_blank(&row.detail);
        view! {
            <div class="space-y-0.5 text-xs">
                {label.map(|l| view! { <div class="font-medium">{l}</div> })}
                <div class="font-mono text-[11px] text-muted-foreground break-all">
                    {element}{page.map(|p| format!(" @ {p}"))}
                </div>
                {detail.map(|d| view! { <div class="text-muted-foreground break-all">{d}</div> })}
            </div>
        }
        .into_any()
    };

    view! {
        <TableRow class="hover:bg-muted/30 transition-colors">
            <TableCell class="text-xs">
                <div title=time_title>
                {match time {
                    Some(t) => view! {
                        <div class="font-mono">{t.time}</div>
                        <div class="text-[11px] text-muted-foreground">{t.date}</div>
                    }.into_any(),
                    None => view! { <div class="font-mono text-[11px]">{row.timestamp.clone()}</div> }.into_any(),
                }}
                </div>
            </TableCell>
            <TableCell>
                <Badge variant=kind_variant(&row.kind) class="text-xs">{kind_label(&row.kind)}</Badge>
            </TableCell>
            <TableCell class="text-xs">
                <div class="font-medium">{username.unwrap_or_else(|| "-".to_string())}</div>
                {user_id.map(|id| view! { <div class="font-mono text-[10px] text-muted-foreground break-all">{id}</div> })}
            </TableCell>
            <TableCell class="text-xs">
                {program.map(|p| view! { <Badge variant=BadgeVariant::Outline class="text-xs font-mono">{p}</Badge> })}
                {action.map(|a| view! { <div class="mt-0.5 text-muted-foreground break-all">{a}</div> })}
            </TableCell>
            <TableCell>{what}</TableCell>
            <TableCell class="text-center">
                {row.status.filter(|_| is_api).map(|s| view! {
                    <Badge variant=status_variant(s) class="text-xs font-mono">{s.to_string()}</Badge>
                })}
            </TableCell>
            <TableCell class="text-right font-mono text-xs text-muted-foreground">
                {row.duration_ms.map(|d| d.to_string())}
            </TableCell>
            <TableCell class="text-right">
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                    on_click=Callback::new(move |_| ctx.open_detail.run(for_open.clone()))
                >
                    {t_or("alpg000.action.details", "ดู")}
                </Button>
            </TableCell>
        </TableRow>
    }
}
