//! Postal code table (shared `DataTable` shell): code, sub-district, district, province, status,
//! actions; search by code or area name, status chips and paging in the header/footer.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg005::providers::use_dbpg005;

const MAIL_ICON: &str = "M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2zM22 6l-10 7L2 6";

#[component]
pub fn PostalTable() -> impl IntoView {
    let ctx = use_dbpg005();
    let can_write = can("DBPG005", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg005.column.postal_code", "รหัสไปรษณีย์")).class("w-36"),
        DataColumn::new(t_or("dbpg005.level.sub_district", "ตำบล / แขวง")),
        DataColumn::new(t_or("dbpg005.level.district", "อำเภอ / เขต")),
        DataColumn::new(t_or("dbpg005.level.province", "จังหวัด")),
        DataColumn::new(t_or("dbpg005.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg005.column.actions", "จัดการ")).class("w-24 text-right"),
    ];
    let dash = |s: String| if s.is_empty() { "-".to_string() } else { s };

    view! {
        <DataTable
            title=t_or("dbpg005.postal.title", "ตารางรหัสไปรษณีย์ (Postal Codes)")
            description=t_or("dbpg005.postal.description", "ค้นหาด้วยรหัสไปรษณีย์ หรือชื่อตำบล/อำเภอ แล้วดูพื้นที่ที่ผูกอยู่")
            icon_path=MAIL_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.postal_page.get().items.is_empty())
            loading_text=t_or("dbpg005.postal.loading", "กำลังโหลดรหัสไปรษณีย์...")
            empty_title=t_or("dbpg005.postal.empty_title", "ไม่พบรหัสไปรษณีย์")
            empty_text=t_or("dbpg005.table.empty_text", "ไม่พบรายการที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg005.postal.search_placeholder", "พิมพ์รหัสไปรษณีย์ หรือชื่อตำบล/อำเภอ...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.postal_page.get().page)
            limit=Signal::derive(move || ctx.postal_page.get().limit)
            total=Signal::derive(move || ctx.postal_page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="DBPG005" action=Action::Write>
                <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {t_or("dbpg005.postal.create", "+ เพิ่มรหัสไปรษณีย์")}
                </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("dbpg005.filter.status", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("dbpg005.filter.status_all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("dbpg005.filter.status_active", "เปิดใช้งาน")),
                        ("INACTIVE".to_string(), t_or("dbpg005.filter.status_inactive", "ปิดใช้งาน")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=ctx.set_status
                />
            }
        >
            {move || {
                let page = ctx.postal_page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, p)| {
                    let for_edit = p.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell class="font-mono font-semibold text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{p.postal_code.clone()}</span>
                            </TableCell>
                            <TableCell class="text-xs text-foreground">{dash(p.sub_district_name.clone())}</TableCell>
                            <TableCell class="text-xs text-foreground">{dash(p.district_name.clone())}</TableCell>
                            <TableCell class="text-xs text-foreground">{p.province_name.clone()}</TableCell>
                            <TableCell class="text-center">
                                {if p.active {
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
                                    on_click=Callback::new(move |_| ctx.open_edit_postal.run(for_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("dbpg005.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                            </TableCell>
                        </TableRow>
                    }
                }).collect_view()
            }}
        </DataTable>
    }
}
