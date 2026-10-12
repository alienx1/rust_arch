//! Program table (shared `DataTable` shell): code, names, system/module, parent, route/icon,
//! order, status, actions; search, status chips, add button and server-side paging.
//!
//! This is the switchboard of the sidebar: a row's status decides whether its menu entry exists.

use leptos::prelude::*;
use crate::access::{Action, PermissionGate, can};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    TableCell, TableRow, data_row_number,
};

use crate::pages::su::supg002::providers::use_supg002;

const MENU_ICON: &str = "M3 12h18M3 6h18M3 18h18";

#[component]
pub fn ProgramTable() -> impl IntoView {
    let ctx = use_supg002();
    let can_write = can("SUPG002", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg002.column.code", "รหัส")).class("w-36"),
        DataColumn::new(t_or("supg002.column.name_one", "ชื่อเมนู")).class("min-w-[200px]"),
        DataColumn::new(t_or("supg002.column.system_module", "ระบบ / โมดูล")),
        DataColumn::new(t_or("supg002.column.parent", "โฟลเดอร์แม่")),
        DataColumn::new(t_or("supg002.column.route_icon", "เส้นทาง / ไอคอน")),
        DataColumn::new(t_or("supg002.column.sort_order", "ลำดับ")).class("w-20 text-center"),
        DataColumn::new(t_or("supg002.column.menu", "เมนู")).class("w-28 text-center"),
        DataColumn::new(t_or("supg002.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("supg002.table.title", "ตารางโปรแกรมและเมนู (Programs)")
            description=t_or("supg002.table.description", "อ้างอิงตาราง: db_program & db_program_label • เปิด/ปิดสถานะเพื่อแสดง/ซ่อนเมนูใน sidebar (ปิดโฟลเดอร์ = ซ่อนทั้งกลุ่ม)")
            icon_path=MENU_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("supg002.table.loading", "กำลังโหลดโปรแกรม...")
            empty_title=t_or("supg002.table.empty_title", "ไม่พบโปรแกรม")
            empty_text=t_or("supg002.table.empty_text", "ไม่พบโปรแกรมที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("supg002.search.placeholder", "ค้นหารหัส, เส้นทาง หรือชื่อเมนู...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="SUPG002" action=Action::Write>
                    <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {move || t_or("supg002.action.create", "+ เพิ่มโปรแกรม")}
                </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("supg002.filter.status", "สถานะเมนู:")
                    options=vec![
                        ("ALL".to_string(), t_or("supg002.filter.all", "ทั้งหมด").to_string()),
                        ("ACTIVE".to_string(), t_or("supg002.status.shown", "แสดง").to_string()),
                        ("INACTIVE".to_string(), t_or("supg002.status.hidden", "ซ่อน").to_string()),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=ctx.set_status
                />
            }
        >
            {move || {
                let page = ctx.page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, p)| {
                    let for_edit = p.clone();
                    let route = p.path.clone().unwrap_or_default();
                    let icon = p.icon.clone().unwrap_or_default();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{p.code.clone()}</span>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-semibold text-foreground flex items-center gap-1.5">
                                        {web_package::core::i18n::localized(&p.name_th, &p.name_en)}
                                        {(p.child_count > 0).then(|| view! {
                                            <Badge variant=BadgeVariant::Outline class="text-[10px]">
                                                {t_fmt("supg002.table.folder_items", "โฟลเดอร์ • {count} รายการ", &[("count", &p.child_count.to_string())])}
                                            </Badge>
                                        })}
                                        {p.is_protected.then(|| view! { <span title=t_or("supg002.table.protected", "หน้าจัดการเมนู/สิทธิ์: ปิดไม่ได้")>"🔒"</span> })}
                                    </div>
                                </div>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5 font-mono">
                                    <div class="text-foreground">{p.system_code.clone()}</div>
                                    <div class="text-[11px] text-muted-foreground">{p.module_code.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell class="text-xs font-mono text-muted-foreground">
                                {if p.parent_code.is_empty() { "-".to_string() } else { p.parent_code.clone() }}
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-mono text-foreground">
                                        {if route.is_empty() { t_or("supg002.table.no_page", "— (ไม่มีหน้า)") } else { route }}
                                    </div>
                                    <div class="text-[11px] text-muted-foreground">
                                        {if icon.is_empty() { "-".to_string() } else { icon }}
                                    </div>
                                </div>
                            </TableCell>
                            <TableCell class="text-center font-mono text-xs">{p.sort_order}</TableCell>
                            <TableCell class="text-center">
                                {if p.active {
                                    view! { <Badge variant=BadgeVariant::Success class="text-xs">{t_or("supg002.status.shown", "แสดง")}</Badge> }
                                } else {
                                    view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">{t_or("supg002.status.hidden", "ซ่อน")}</Badge> }
                                }}
                            </TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit.run(for_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("common.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                            </TableCell>
                        </TableRow>
                    }
                }).collect_view()
            }}
        </DataTable>
    }
}
