//! Lookup table (shared `DataTable` shell): category, code, names, status, actions; search, status
//! chips, refresh, add button and server-side paging live in the header/footer.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg004::providers::use_dbpg004;

const BOOK_ICON: &str = "M4 19.5A2.5 2.5 0 0 1 6.5 17H20M4 19.5A2.5 2.5 0 0 0 6.5 22H20V2H6.5A2.5 2.5 0 0 0 4 4.5z";

#[component]
pub fn LookupTable() -> impl IntoView {
    let ctx = use_dbpg004();
    let can_write = can("DBPG004", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg004.column.category", "หมวดหมู่")).class("w-44"),
        DataColumn::new(t_or("dbpg004.column.code", "รหัส")).class("w-32"),
        DataColumn::new(t_or("dbpg004.column.name", "ชื่อ")).class("min-w-[180px]"),
        DataColumn::new(t_or("dbpg004.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg004.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg004.table.title", "ตารางข้อมูลอ้างอิง (Reference Data)")
            description=t_or("dbpg004.table.description", "อ้างอิงตาราง: db_master_lookup & db_master_lookup_label • ค้นหา/กรอง/แบ่งหน้าที่ฐานข้อมูล")
            icon_path=BOOK_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("dbpg004.table.loading", "กำลังโหลดข้อมูลอ้างอิง...")
            empty_title=t_or("dbpg004.table.empty_title", "ไม่พบข้อมูลอ้างอิง")
            empty_text=t_or("dbpg004.table.empty_text", "ไม่พบค่าอ้างอิงที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg004.search.placeholder", "ค้นหารหัส หรือชื่อ...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="DBPG004" action=Action::Write>
                <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {move || t_or("dbpg004.action.create", "+ เพิ่มค่าอ้างอิง")}
                </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("dbpg004.filter.status", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("dbpg004.filter.status_all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("dbpg004.filter.status_active", "เปิดใช้งาน")),
                        ("INACTIVE".to_string(), t_or("dbpg004.filter.status_inactive", "ปิดใช้งาน")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=ctx.set_status
                />
            }
        >
            {move || {
                let page = ctx.page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, l)| {
                    let for_edit = l.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell>
                                <Badge variant=BadgeVariant::Outline class="text-xs font-mono">
                                    {l.category.clone()}
                                </Badge>
                            </TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{l.code.clone()}</span>
                            </TableCell>
                            <TableCell class="font-semibold text-foreground text-xs">
                                <div class="flex items-center gap-1.5">
                                    {localized(&l.name_th, &l.name_en)}
                                    {l.is_protected.then(|| view! {
                                        <span title=t_or("dbpg004.table.protected_hint", "ค่าหลักของระบบ: ปิดใช้งานไม่ได้")>"🔒"</span>
                                    })}
                                </div>
                            </TableCell>
                            <TableCell class="text-center">
                                {if l.active {
                                    view! { <Badge variant=BadgeVariant::Success class="text-xs">"Active"</Badge> }
                                } else {
                                    view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">"Inactive"</Badge> }
                                }}
                            </TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit.run(for_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("dbpg004.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                            </TableCell>
                        </TableRow>
                    }
                }).collect_view()
            }}
        </DataTable>
    }
}
