//! My reports (shared `DataTable` shell): requested time, report, status, rows, size, actions.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, TableCell, TableRow,
};

use crate::pages::al::alpg000::components::audit_time_format::EventTime;
use crate::pages::rp::rppg000::models::{
    ReportJobDto, STATUS_FAILED, STATUS_RUNNING, STATUS_SUCCEEDED, size_text,
};
use crate::pages::rp::rppg000::providers::use_rppg000;

const FILE_ICON: &str = "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z M14 2v6h6";

fn status_label(status: &str) -> String {
    match status {
        STATUS_RUNNING => t_or("rppg000.status.running", "กำลังสร้าง"),
        STATUS_SUCCEEDED => t_or("rppg000.status.succeeded", "พร้อมดาวน์โหลด"),
        STATUS_FAILED => t_or("rppg000.status.failed", "ไม่สำเร็จ"),
        other => other.to_string(),
    }
}

fn status_variant(status: &str) -> BadgeVariant {
    match status {
        STATUS_SUCCEEDED => BadgeVariant::Success,
        STATUS_FAILED => BadgeVariant::Destructive,
        _ => BadgeVariant::Warning,
    }
}

fn kind_label(kind: &str) -> String {
    match kind {
        "audit_log_csv" => t_or("rppg000.kind.audit_log_csv", "บันทึกเหตุการณ์ (CSV)"),
        other => other.to_string(),
    }
}

#[component]
pub fn ReportJobTable() -> impl IntoView {
    let ctx = use_rppg000();
    let columns = vec![
        DataColumn::new(t_or("rppg000.column.requested", "เวลาที่ขอ")).class("w-40"),
        DataColumn::new(t_or("rppg000.column.report", "รายงาน")).class("min-w-[220px]"),
        DataColumn::new(t_or("rppg000.column.status", "สถานะ")).class("w-36"),
        DataColumn::new(t_or("rppg000.column.rows", "จำนวนแถว")).class("w-28 text-right"),
        DataColumn::new(t_or("rppg000.column.size", "ขนาด")).class("w-24 text-right"),
        DataColumn::new(t_or("rppg000.column.actions", "จัดการ")).class("w-44 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("rppg000.table.title", "รายงานของฉัน")
            description=t_or("rppg000.table.description", "รายงานที่ขอไว้ ระบบสร้างให้เบื้องหลังและแจ้งเตือนเมื่อพร้อม • เก็บเป็นไฟล์ใน Object Storage")
            icon_path=FILE_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.jobs.get().is_empty())
            loading_text=t_or("rppg000.state.loading", "กำลังโหลดรายการรายงาน...")
            empty_title=t_or("rppg000.state.empty_title", "ยังไม่มีรายงาน")
            empty_text=t_or("rppg000.state.empty_text", "กด \"ส่งออก CSV\" ที่หน้าบันทึกเหตุการณ์เพื่อขอรายงาน")
            on_refresh=ctx.refresh
            notice=move || view! {
                {move || ctx.error.get().map(|e| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">{e}</div>
                })}
            }
        >
            {move || ctx.jobs.get().into_iter().map(|job| view! { <ReportJobRow job=job /> }).collect_view()}
        </DataTable>
    }
}

#[component]
fn ReportJobRow(job: ReportJobDto) -> impl IntoView {
    let ctx = use_rppg000();
    let time = EventTime::parse(&job.requested_at);
    let ready = job.status == STATUS_SUCCEEDED;
    let for_preview = job.clone();
    let for_download = job.clone();

    view! {
        <TableRow class="hover:bg-muted/30 transition-colors">
            <TableCell class="text-xs">
                {match time {
                    Some(t) => view! {
                        <div class="font-mono">{t.time}</div>
                        <div class="text-[11px] text-muted-foreground">{t.date}</div>
                    }.into_any(),
                    None => view! { <div class="font-mono text-[11px]">{job.requested_at.clone()}</div> }.into_any(),
                }}
            </TableCell>
            <TableCell class="text-xs">
                <div class="font-medium">{kind_label(&job.report_kind)}</div>
                <div class="font-mono text-[11px] text-muted-foreground break-all">{job.file_name.clone().unwrap_or_default()}</div>
                {job.error.clone().map(|e| view! { <div class="text-destructive break-all">{e}</div> })}
            </TableCell>
            <TableCell>
                <Badge variant=status_variant(&job.status) class="text-xs">{status_label(&job.status)}</Badge>
            </TableCell>
            <TableCell class="text-right font-mono text-xs">
                {ready.then(|| job.row_count.to_string())}
            </TableCell>
            <TableCell class="text-right font-mono text-xs text-muted-foreground">
                {ready.then(|| size_text(job.size_bytes))}
            </TableCell>
            <TableCell class="text-right">
                {ready.then(|| view! {
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Sm
                        class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                        on_click=Callback::new(move |_| ctx.open_preview.run(for_preview.clone()))
                    >
                        {t_or("rppg000.action.preview", "ดูตัวอย่าง")}
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        class="h-7 px-2 text-xs ml-1"
                        on_click=Callback::new(move |_| ctx.download.run(for_download.clone()))
                    >
                        {t_or("rppg000.action.download", "ดาวน์โหลด")}
                    </Button>
                })}
            </TableCell>
        </TableRow>
    }
}
