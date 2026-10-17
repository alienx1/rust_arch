//! All notifications (shared `DataTable` shell): time, what, status; click opens the thing the
//! notification points at.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{ConfirmButton, 
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, ClientPaging, DataColumn, DataTable, FilterChips,
    TableCell, TableRow,
};

use crate::pages::nt::ntpg000::components::notification_bell::{dot_class, local_time};
use crate::pages::nt::ntpg000::models::NotificationDto;
use crate::pages::nt::ntpg000::providers::use_ntpg000;

const STATUS_ALL: &str = "ALL";
const STATUS_UNREAD: &str = "UNREAD";
const STATUS_READ: &str = "READ";

const BELL_ICON: &str = "M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0";

#[component]
pub fn NotificationTable() -> impl IntoView {
    let ctx = use_ntpg000();
    let paging = ClientPaging::new(10);
    let (search, set_search) = signal(String::new());
    let (status, set_status) = signal(STATUS_ALL.to_string());

    let filtered = Memo::new(move |_| {
        let q = search.get().trim().to_lowercase();
        let st = status.get();
        ctx.items
            .get()
            .into_iter()
            .filter(|n| match st.as_str() {
                STATUS_UNREAD => !n.read,
                STATUS_READ => n.read,
                _ => true,
            })
            .filter(|n| q.is_empty() || n.title_text().to_lowercase().contains(&q) || n.body_text().to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    });
    let total = Signal::derive(move || filtered.with(|l| l.len() as u64));

    let columns = vec![
        DataColumn::new("#").class("w-12 text-center"),
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
            empty=Signal::derive(move || total.get() == 0)
            loading_text=t_or("ntpg000.state.loading", "กำลังโหลดการแจ้งเตือน...")
            empty_title=t_or("ntpg000.state.empty_title", "ยังไม่มีการแจ้งเตือน")
            empty_text=t_or("ntpg000.state.empty_text", "เมื่อมีเรื่องที่ต้องแจ้ง จะแสดงที่นี่และที่ไอคอนกระดิ่ง")
            on_refresh=ctx.refresh
            search_value=Signal::derive(move || search.get())
            search_placeholder=t_or("ntpg000.search.placeholder", "ค้นหาหัวข้อหรือข้อความ...")
            on_search=Callback::new(move |v: String| {
                paging.set_page(1);
                set_search.set(v);
            })
            filters=move || view! {
                <FilterChips
                    label=t_or("ntpg000.filter.status", "สถานะ:")
                    options=vec![
                        (STATUS_ALL.to_string(), t_or("ntpg000.filter.all", "ทั้งหมด")),
                        (STATUS_UNREAD.to_string(), t_or("ntpg000.status.unread", "ใหม่")),
                        (STATUS_READ.to_string(), t_or("ntpg000.status.read", "อ่านแล้ว")),
                    ]
                    value=Signal::derive(move || status.get())
                    on_change=Callback::new(move |v: String| {
                        paging.set_page(1);
                        set_status.set(v);
                    })
                />
            }
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            actions=move || view! {
                <ConfirmButton
                    label=Signal::derive(move || t_or("ntpg000.bell.mark_read", "อ่านทั้งหมดแล้ว"))
                    confirm_text=Signal::derive(move || t_or("ntpg000.bell.mark_read_confirm", "ทำเครื่องหมายว่าอ่านทุกรายการแล้ว?"))
                    class="h-9 text-xs"
                    on_confirm=Callback::new(move |_| ctx.mark_all_read.run(()))
                />
            }
            notice=move || view! {
                {move || ctx.error.get().map(|e| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">{e}</div>
                })}
            }
        >
            {move || {
                let (rows, first) = paging.slice(filtered.get());
                rows.into_iter()
                    .enumerate()
                    .map(|(i, n)| view! { <NotificationRow n=n number=first + i as u64 /> })
                    .collect_view()
            }}
        </DataTable>
    }
}

#[component]
fn NotificationRow(n: NotificationDto, number: u64) -> impl IntoView {
    let ctx = use_ntpg000();
    let (dot, time) = (dot_class(&n.kind), local_time(&n.created_at));
    let unread = !n.read;
    let for_open = n.clone();
    let (title_n, body_n) = (n.clone(), n.clone());
    view! {
        <TableRow class=if unread { "bg-[color-mix(in_oklch,var(--primary)_12%,transparent)] hover:bg-[color-mix(in_oklch,var(--primary)_20%,transparent)] transition-colors" } else { "hover:bg-muted/30 transition-colors" }>
            // The accent bar sits on the first cell (a row cannot carry its own border in a
            // collapsed table); read rows keep a transparent bar so nothing shifts
            <TableCell class=if unread { "text-center font-mono text-xs font-semibold border-l-4 border-l-primary" } else { "text-center font-mono text-xs text-muted-foreground border-l-4 border-l-transparent" }>{number}</TableCell>
            <TableCell class=if unread { "text-xs font-mono font-semibold" } else { "text-xs font-mono text-muted-foreground" }>{time}</TableCell>
            <TableCell class="text-xs">
                <div class="flex items-start gap-2">
                    <span class=format!("mt-1.5 h-2 w-2 shrink-0 rounded-full {dot}")></span>
                    <div>
                        <div class=if unread { "font-bold text-foreground" } else { "font-medium text-muted-foreground" }>{move || title_n.title_text()}</div>
                        <div class=if unread { "text-foreground/80 break-all" } else { "text-muted-foreground break-all" }>{move || body_n.body_text()}</div>
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
