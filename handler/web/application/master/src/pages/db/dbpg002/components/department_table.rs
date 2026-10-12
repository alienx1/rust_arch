//! Department table (shared `DataTable` shell): code, names, branch, company, status, actions;
//! search, status chips and refresh live in the header.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, ClientPaging, FilterChips,
    TableCell, TableRow,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg002::providers::use_dbpg002;

const GRID_ICON: &str = "M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z";

#[component]
pub fn DepartmentTable() -> impl IntoView {
    let ctx = use_dbpg002();
    let can_write = can("DBPG002", Action::Write);

    let paging = ClientPaging::new(10);
    let total = Signal::derive(move || ctx.departments.get().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg002.dept.column.code", "รหัสแผนก")).class("w-28"),
        DataColumn::new(t_or("dbpg002.dept.column.name", "ชื่อแผนก")).class("min-w-[200px]"),
        DataColumn::new(t_or("dbpg002.dept.column.branch", "สาขา")),
        DataColumn::new(t_or("dbpg002.dept.column.company", "บริษัท")),
        DataColumn::new(t_or("dbpg002.dept.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg002.dept.column.actions", "จัดการ")).class("w-28 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg002.dept.table.title", "ตารางแผนก (Department Directory)")
            description=t_or("dbpg002.dept.table.description", "อ้างอิงตาราง: db_department & db_department_label")
            icon_path=GRID_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.departments.get().is_empty())
            loading_text=t_or("dbpg002.dept.table.loading", "กำลังโหลดข้อมูลแผนก...")
            empty_title=t_or("dbpg002.dept.table.empty_title", "ไม่พบข้อมูลแผนก")
            empty_text=t_or("dbpg002.dept.table.empty_text", "ไม่พบแผนกที่ตรงกับเงื่อนไข หรือยังไม่มีแผนก (ต้องมีสาขาก่อนจึงจะเพิ่มแผนกได้)")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg002.dept.search.placeholder", "ค้นหารหัส, ชื่อแผนก, สาขา...")
            on_search=Callback::new(move |v| ctx.set_search_query.set(v))
            on_refresh=ctx.refresh
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            empty_action=move || view! {
                <PermissionGate program="DBPG002" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        class="text-xs"
                        on_click=Callback::new(move |_| ctx.open_create.run(()))
                    >
                        {move || t_or("dbpg002.dept.action.create", "+ เพิ่มแผนกใหม่")}
                    </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("dbpg002.filter.status_label", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("dbpg002.filter.all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("common.status.active", "เปิดใช้งาน")),
                        ("INACTIVE".to_string(), t_or("dbpg002.filter.inactive", "ปิดใช้งาน")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=Callback::new(move |v| ctx.set_status_filter.set(v))
                />
            }
        >
            {move || {
                let (items, first) = paging.slice(ctx.departments.get());
                items.into_iter().enumerate().map(|(idx, d)| {
                let for_edit = d.clone();
                view! {
                    <TableRow class="hover:bg-muted/30 transition-colors">
                        <TableCell class="text-center font-mono text-xs text-muted-foreground">
                            {first + idx as u64}
                        </TableCell>
                        <TableCell class="font-mono font-medium text-xs">
                            <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{d.code.clone()}</span>
                        </TableCell>
                        <TableCell>
                            <div class="text-xs font-semibold text-foreground">
                                {let n = localized(&d.name_th, &d.name_en); if n.is_empty() { "-".to_string() } else { n }}
                            </div>
                        </TableCell>
                        <TableCell>
                            <div class="text-xs space-y-0.5">
                                <div class="text-foreground">{d.branch_name.clone()}</div>
                                <div class="text-[11px] font-mono text-muted-foreground">{d.branch_code.clone()}</div>
                            </div>
                        </TableCell>
                        <TableCell>
                            <div class="text-xs space-y-0.5">
                                <div class="text-foreground">{d.company_name.clone()}</div>
                                <div class="text-[11px] font-mono text-muted-foreground">{d.company_code.clone()}</div>
                            </div>
                        </TableCell>
                        <TableCell class="text-center">
                            {if d.active {
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
                                on_click=Callback::new(move |_| ctx.open_edit_department.run(for_edit.clone()))
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
