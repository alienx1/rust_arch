//! Process list (shared `DataTable` shell): requested time, process, requester, status, outcome,
//! run time and the reprocess action.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{ConfirmButton, 
    Badge, BadgeVariant, ButtonVariant, DataColumn, DataTable, TableCell, TableRow,
};

use crate::pages::al::alpg000::components::audit_time_format::EventTime;
use crate::pages::al::alpg001::components::ProcessFilterBar;
use crate::pages::al::alpg001::models::{
    ProcessDto, ProcessFilterField, STATUS_FAILED, STATUS_QUEUED, STATUS_RUNNING, STATUS_SUCCEEDED,
    duration_text, size_text,
};
use crate::pages::al::alpg001::providers::use_alpg001;

const SYNC_ICON: &str = "M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8 M3 3v5h5 M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16 M16 16h5v5";

fn status_label(status: &str) -> String {
    match status {
        STATUS_QUEUED => t_or("alpg001.status.queued", "รอคิว"),
        STATUS_RUNNING => t_or("alpg001.status.running", "กำลังทำงาน"),
        STATUS_SUCCEEDED => t_or("alpg001.status.succeeded", "สำเร็จ"),
        STATUS_FAILED => t_or("alpg001.status.failed", "ไม่สำเร็จ"),
        other => other.to_string(),
    }
}

fn status_variant(status: &str) -> BadgeVariant {
    match status {
        STATUS_SUCCEEDED => BadgeVariant::Success,
        STATUS_FAILED => BadgeVariant::Destructive,
        STATUS_RUNNING => BadgeVariant::Warning,
        _ => BadgeVariant::Secondary,
    }
}

fn kind_label(kind: &str) -> String {
    match kind {
        "audit_log_csv" => t_or("alpg001.kind.audit_log_csv", "ส่งออกบันทึกเหตุการณ์ (CSV)"),
        other => other.to_string(),
    }
}

#[component]
pub fn ProcessTable() -> impl IntoView {
    let ctx = use_alpg001();
    let columns = vec![
        DataColumn::new(t_or("alpg001.column.requested", "เวลาที่ขอ")).class("w-36"),
        DataColumn::new(t_or("alpg001.column.process", "กระบวนการ")).class("min-w-[200px]"),
        DataColumn::new(t_or("alpg001.column.user", "ผู้ขอ")).class("w-32"),
        DataColumn::new(t_or("alpg001.column.status", "สถานะ")).class("w-32"),
        DataColumn::new(t_or("alpg001.column.result", "ผลลัพธ์")).class("min-w-[220px]"),
        DataColumn::new(t_or("alpg001.column.duration", "ใช้เวลา")).class("w-24 text-right"),
        DataColumn::new(t_or("alpg001.column.actions", "จัดการ")).class("w-36 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("alpg001.table.title", "กระบวนการเบื้องหลัง (Process Monitor)")
            description=t_or("alpg001.table.description", "งานที่ส่งผ่าน Redpanda (เช่น สร้างรายงาน CSV) สำเร็จหรือไม่ พร้อมสั่งประมวลผลซ้ำเมื่อไม่สำเร็จหรือค้าง")
            icon_path=SYNC_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("alpg001.state.loading", "กำลังโหลดรายการกระบวนการ...")
            empty_title=t_or("alpg001.state.empty_title", "ไม่พบกระบวนการ")
            empty_text=t_or("alpg001.state.empty_text", "ไม่พบรายการที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.filters.get().user)
            search_placeholder=t_or("alpg001.search.placeholder", "ค้นหาชื่อผู้ใช้ที่ขอ...")
            on_search=Callback::new(move |v: String| ctx.set_filter.run((ProcessFilterField::User, v)))
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            notice=move || view! {
                {move || ctx.error.get().map(|e| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">{e}</div>
                })}
                {move || ctx.notice.get().map(|(ok, text)| view! {
                    <div class=if ok {
                        "p-3 rounded-lg border border-border bg-muted/40 text-xs font-medium"
                    } else {
                        "p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium"
                    }>{text}</div>
                })}
            }
            filters=move || view! { <ProcessFilterBar /> }
        >
            {move || ctx.page.get().items.into_iter().map(|p| view! { <ProcessRow process=p /> }).collect_view()}
        </DataTable>
    }
}

#[component]
fn ProcessRow(process: ProcessDto) -> impl IntoView {
    let ctx = use_alpg001();
    let time = EventTime::parse(&process.requested_at);
    let id = process.id.clone();
    let for_action = process.clone();
    let is_busy = {
        let id = id.clone();
        move || ctx.busy.get().as_deref() == Some(id.as_str())
    };
    let ok = process.status == STATUS_SUCCEEDED;

    view! {
        <TableRow class="hover:bg-muted/30 transition-colors">
            <TableCell class="text-xs">
                {match time {
                    Some(t) => view! {
                        <div class="font-mono">{t.time}</div>
                        <div class="text-[11px] text-muted-foreground">{t.date}</div>
                    }.into_any(),
                    None => view! { <div class="font-mono text-[11px]">{process.requested_at.clone()}</div> }.into_any(),
                }}
            </TableCell>
            <TableCell class="text-xs">
                <div class="font-medium">{kind_label(&process.kind)}</div>
                <div class="font-mono text-[10px] text-muted-foreground break-all">{process.id.clone()}</div>
            </TableCell>
            <TableCell class="text-xs">{process.requested_by.clone()}</TableCell>
            <TableCell>
                <Badge variant=status_variant(&process.status) class="text-xs">{status_label(&process.status)}</Badge>
                {process.stuck.then(|| view! {
                    <div class="mt-1 text-[11px] text-destructive font-medium">
                        {t_or("alpg001.status.stuck", "ค้างนานผิดปกติ")}
                    </div>
                })}
            </TableCell>
            <TableCell class="text-xs">
                {match (&process.error, ok) {
                    (Some(e), _) => view! {
                        <div class="text-destructive break-words line-clamp-3" title=e.clone()>{e.clone()}</div>
                    }.into_any(),
                    (None, true) => view! {
                        <div class="font-mono">
                            {format!("{} {} · {}", process.row_count, t_or("alpg001.unit.rows", "แถว"), size_text(process.size_bytes))}
                        </div>
                        <div class="font-mono text-[11px] text-muted-foreground break-all">{process.file_name.clone().unwrap_or_default()}</div>
                    }.into_any(),
                    _ => ().into_any(),
                }}
            </TableCell>
            <TableCell class="text-right font-mono text-xs text-muted-foreground">
                {process.duration_ms.map(duration_text)}
            </TableCell>
            <TableCell class="text-right">
                {process.can_reprocess.then(|| view! {
                    <ConfirmButton
                        label=Signal::derive(move || t_or("alpg001.action.reprocess", "ประมวลผลใหม่"))
                        confirm_text=Signal::derive(move || t_or("alpg001.action.reprocess_confirm", "ประมวลผลรายการนี้ใหม่?"))
                        // A failed or stuck process is the one to act on; re-running a good one is secondary
                        variant=if ok { ButtonVariant::Ghost } else { ButtonVariant::Outline }
                        class="h-7 px-2 text-xs"
                        busy=Signal::derive(is_busy.clone())
                        on_confirm=Callback::new(move |_| ctx.reprocess.run(for_action.clone()))
                    />
                })}
            </TableCell>
        </TableRow>
    }
}
