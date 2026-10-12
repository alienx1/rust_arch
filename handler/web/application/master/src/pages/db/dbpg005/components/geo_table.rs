//! Province / district / sub-district table (shared `DataTable` shell). Columns follow the level;
//! search, status chips, refresh, add button and server-side paging live in the header/footer.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg005::models::GeoLevel;
use crate::pages::db::dbpg005::providers::use_dbpg005;

const MAP_ICON: &str = "M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0zM12 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6z";

fn columns_for(level: GeoLevel) -> Vec<DataColumn> {
    let mut cols = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg005.column.code", "รหัส")).class("w-28"),
        DataColumn::new(t_or("dbpg005.column.name_single", "ชื่อ")).class("min-w-[200px]"),
    ];
    match level {
        GeoLevel::District => cols.push(DataColumn::new(t_or("dbpg005.column.province", "จังหวัด"))),
        GeoLevel::SubDistrict => {
            cols.push(DataColumn::new(t_or("dbpg005.column.district_province", "อำเภอ / จังหวัด")));
            cols.push(DataColumn::new(t_or("dbpg005.column.postal_code", "รหัสไปรษณีย์")));
        }
        _ => {}
    }
    cols.push(DataColumn::new(t_or("dbpg005.column.status", "สถานะ")).class("w-28 text-center"));
    cols.push(DataColumn::new(t_or("dbpg005.column.actions", "จัดการ")).class("w-24 text-right"));
    cols
}

/// `level` must be `Province`, `District` or `SubDistrict` (postal codes have their own table)
#[component]
pub fn GeoTable(level: GeoLevel) -> impl IntoView {
    let ctx = use_dbpg005();
    let can_write = can("DBPG005", Action::Write);

    view! {
        <DataTable
            title=t_fmt("dbpg005.table.title", "ตาราง{level} ", &[("level", level.label().as_str())])
            description=t_or("dbpg005.table.description", "ค้นหา/กรอง/แบ่งหน้าที่ฐานข้อมูล • ข้อมูลอ้างอิงภูมิศาสตร์สำหรับที่อยู่")
            icon_path=MAP_ICON
            columns=columns_for(level)
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.geo_page.get().items.is_empty())
            loading_text=t_or("common.state.loading", "กำลังโหลดข้อมูล...")
            empty_title=t_or("common.state.empty", "ไม่พบข้อมูล")
            empty_text=t_or("dbpg005.table.empty_text", "ไม่พบรายการที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg005.search.placeholder", "ค้นหารหัส หรือชื่อ...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.geo_page.get().page)
            limit=Signal::derive(move || ctx.geo_page.get().limit)
            total=Signal::derive(move || ctx.geo_page.get().total)
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
                    {t_fmt("dbpg005.action.create", "+ เพิ่ม{level}", &[("level", level.label().as_str())])}
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
                let page = ctx.geo_page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, n)| {
                    let for_edit = n.clone();

                    // Level-specific cells, built from owned clones
                    let province_cell = (level == GeoLevel::District).then({
                        let province = n.province_name.clone();
                        move || view! {
                            <TableCell class="text-xs text-foreground">{province}</TableCell>
                        }
                    });
                    let parents_cell = (level == GeoLevel::SubDistrict).then({
                        let (district, province) = (n.district_name.clone(), n.province_name.clone());
                        move || view! {
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="text-foreground">{district}</div>
                                    <div class="text-[11px] text-muted-foreground">{province}</div>
                                </div>
                            </TableCell>
                        }
                    });
                    let postal_cell = (level == GeoLevel::SubDistrict).then({
                        let codes = n.postal_codes.clone();
                        move || view! {
                            <TableCell>
                                <div class="flex flex-wrap gap-1">
                                    {codes.into_iter().map(|p| view! {
                                        <Badge variant=BadgeVariant::Outline class="text-[10px] font-mono">{p}</Badge>
                                    }).collect_view()}
                                </div>
                            </TableCell>
                        }
                    });

                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{n.code.clone()}</span>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs font-semibold text-foreground">{localized(&n.name_th, &n.name_en)}</div>
                            </TableCell>
                            {province_cell}
                            {parents_cell}
                            {postal_cell}
                            <TableCell class="text-center">
                                {if n.active {
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
                                    on_click=Callback::new(move |_| ctx.open_edit_geo.run(for_edit.clone()))
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
