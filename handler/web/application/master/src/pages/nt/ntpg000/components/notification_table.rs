//! All notifications (shared `DataTable` shell): time, what, status; click opens the thing the
//! notification points at.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, TableCell, TableRow,
};

use crate::pages::nt::ntpg000::components::notification_bell::{dot_class, local_time};
use crate::pages::nt::ntpg000::models::NotificationDto;
use crate::pages::nt::ntpg000::providers::use_ntpg000;

const BELL_ICON: &str = "M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0";

#[component]
pub fn NotificationTable() -> impl IntoView {
    let ctx = use_ntpg000();
    let columns = vec![
        DataColumn::new(t_or("ntpg000.column.time", "เวลา")).class("w-44"),
        DataColumn::new(t_or("ntpg000.column.what", "การแจ้งเตือน")).class("min-w-[260px]"),
        DataColumn::new(t_or("ntpg000.column.status", "สถานะ")).class("w-28"),
        DataColumn::new(t_or("ntpg000.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("ntpg000.table.title", "การแจ้งเตือน")
            description=t_or("ntpg000.table.description", "สิ่งที่ระบบแจ้งให้คุณทราบ (เช่น รายงานที่สร้างเสร็จ) • ส่งแบบเรียลไทม์ไปทุกหน้าจอที่คุณเปิดอยู่")
            icon_path=BELL_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.items.get().is_empty())
            loading_text=t_or("ntpg000.state.loading", "กำลังโหลดการแจ้งเตือน...")
            empty_title=t_or("ntpg000.state.empty_title", "ยังไม่มีการแจ้งเตือน")
            empty_text=t_or("ntpg000.state.empty_text", "เมื่อมีเรื่องที่ต้องแจ้ง จะแสดงที่นี่และที่ไอคอนกระดิ่ง")
            on_refresh=ctx.refresh
            actions=move || view! {
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    class="h-9 text-xs"
                    on_click=Callback::new(move |_| ctx.mark_all_read.run(()))
                >
                    {t_or("ntpg000.bell.mark_read", "อ่านทั้งหมดแล้ว")}
                </Button>
            }
            notice=move || view! {
                {move || ctx.error.get().map(|e| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">{e}</div>
                })}
            }
        >
            {move || ctx.items.get().into_iter().map(|n| view! { <NotificationRow n=n /> }).collect_view()}
        </DataTable>
    }
}

#[component]
fn NotificationRow(n: NotificationDto) -> impl IntoView {
    let ctx = use_ntpg000();
    let (dot, time) = (dot_class(&n.kind), local_time(&n.created_at));
    let unread = !n.read;
    let for_open = n.clone();
    view! {
        <TableRow class=if unread { "bg-[color-mix(in_oklch,var(--primary)_12%,transparent)] hover:bg-[color-mix(in_oklch,var(--primary)_20%,transparent)] transition-colors" } else { "hover:bg-muted/30 transition-colors" }>
            // The accent bar sits on the first cell (a row cannot carry its own border in a
            // collapsed table); read rows keep a transparent bar so nothing shifts
            <TableCell class=if unread { "text-xs font-mono font-semibold border-l-4 border-l-primary" } else { "text-xs font-mono text-muted-foreground border-l-4 border-l-transparent" }>{time}</TableCell>
            <TableCell class="text-xs">
                <div class="flex items-start gap-2">
                    <span class=format!("mt-1.5 h-2 w-2 shrink-0 rounded-full {dot}")></span>
                    <div>
                        <div class=if unread { "font-bold text-foreground" } else { "font-medium text-muted-foreground" }>{n.title.clone()}</div>
                        <div class=if unread { "text-foreground/80 break-all" } else { "text-muted-foreground break-all" }>{n.body.clone()}</div>
                    </div>
                </div>
            </TableCell>
            <TableCell>
                {if unread {
                    view! { <Badge variant=BadgeVariant::Default class="text-xs">{t_or("ntpg000.status.unread", "ใหม่")}</Badge> }.into_any()
                } else {
                    view! { <Badge variant=BadgeVariant::Outline class="text-xs">{t_or("ntpg000.status.read", "อ่านแล้ว")}</Badge> }.into_any()
                }}
            </TableCell>
            <TableCell class="text-right">
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                    on_click=Callback::new(move |_| ctx.open.run(for_open.clone()))
                >
                    {t_or("ntpg000.action.open", "เปิด")}
                </Button>
            </TableCell>
        </TableRow>
    }
}
