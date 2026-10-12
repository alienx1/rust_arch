//! Preview of a finished report: the first rows of the file in a scrollable table.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{Button, ButtonSize, ButtonVariant, Dialog, DialogHeader, DialogTitle, Spinner};

use crate::pages::rp::rppg000::models::size_text;
use crate::pages::rp::rppg000::providers::{PreviewState, use_rppg000};

#[component]
pub fn ReportPreviewDialog() -> impl IntoView {
    let ctx = use_rppg000();
    let open = Signal::derive(move || !matches!(ctx.preview.get(), PreviewState::Closed));

    view! {
        <Dialog open=open on_close=ctx.close_preview class="max-w-6xl w-[95vw]">
            <DialogHeader>
                <DialogTitle>{t_or("rppg000.preview.title", "ตัวอย่างไฟล์รายงาน")}</DialogTitle>
            </DialogHeader>
            {move || match ctx.preview.get() {
                PreviewState::Closed => ().into_any(),
                PreviewState::Loading { .. } => view! {
                    <div class="flex items-center gap-2 text-xs text-muted-foreground py-8 justify-center">
                        <Spinner class="h-4 w-4" />
                        {t_or("rppg000.preview.loading", "กำลังอ่านไฟล์...")}
                    </div>
                }.into_any(),
                PreviewState::Failed { message, .. } => view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">{message}</div>
                }.into_any(),
                PreviewState::Ready { job, preview } => {
                    let shown = preview.rows.len().to_string();
                    let total = preview.total_rows.to_string();
                    let summary = t_fmt(
                        "rppg000.preview.summary",
                        "แสดง {shown} จาก {total} แถว • {size}",
                        &[("shown", &shown), ("total", &total), ("size", &size_text(preview.size_bytes))],
                    );
                    view! {
                        <div class="space-y-3 min-w-0">
                            <div class="flex items-center justify-between gap-3">
                                <div class="text-xs text-muted-foreground">
                                    <span class="font-mono">{preview.file_name.clone()}</span>" • "{summary}
                                </div>
                                <Button
                                    variant=ButtonVariant::Outline
                                    size=ButtonSize::Sm
                                    class="h-8 text-xs"
                                    on_click=Callback::new(move |_| ctx.download.run(job.clone()))
                                >
                                    {t_or("rppg000.action.download", "ดาวน์โหลด")}
                                </Button>
                            </div>
                            <div class="max-h-[60vh] overflow-auto rounded-md border border-border">
                                <table class="w-full text-xs">
                                    <thead class="sticky top-0 bg-muted">
                                        <tr>
                                            {preview.columns.iter().map(|c| view! {
                                                <th class="px-2 py-1.5 text-left font-semibold whitespace-nowrap">{c.clone()}</th>
                                            }).collect_view()}
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {preview.rows.iter().map(|row| view! {
                                            <tr class="border-t border-border/50 hover:bg-muted/30">
                                                {row.iter().map(|cell| view! {
                                                    <td class="px-2 py-1 font-mono whitespace-nowrap max-w-[320px] truncate" title=cell.clone()>{cell.clone()}</td>
                                                }).collect_view()}
                                            </tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </Dialog>
    }
}
