//! Branch table (shared `DataTable` shell): code, names, company, type (headquarter badge),
//! department count, status, actions; search, status chips and refresh live in the header.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, ClientPaging, FilterChips,
    TableCell, TableRow,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg002::providers::use_dbpg002;

const BUILDING_ICON: &str = "M3 21h18M5 21V7l8-4v18M19 21V11l-6-4M9 9v.01M9 12v.01M9 15v.01M9 18v.01";

#[component]
pub fn BranchTable() -> impl IntoView {
    let ctx = use_dbpg002();
    let can_write = can("DBPG002", Action::Write);

    let paging = ClientPaging::new(10);
    let total = Signal::derive(move || ctx.branches.get().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg002.branch.column.code", "รหัสสาขา")).class("w-28"),
        DataColumn::new(t_or("dbpg002.branch.column.name", "ชื่อสาขา")).class("min-w-[200px]"),
        DataColumn::new(t_or("dbpg002.branch.column.company", "บริษัท")),
        DataColumn::new(t_or("dbpg002.branch.column.type", "ประเภท")),
        DataColumn::new(t_or("dbpg002.branch.column.dept", "แผนก")).class("w-24 text-center"),
        DataColumn::new(t_or("dbpg002.branch.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg002.branch.column.actions", "จัดการ")).class("w-44 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg002.branch.table.title", "ตารางสาขา (Branch Directory)")
            description=t_or("dbpg002.branch.table.description", "อ้างอิงตาราง: db_branch & db_branch_label")
            icon_path=BUILDING_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.branches.get().is_empty())
            loading_text=t_or("dbpg002.branch.table.loading", "กำลังโหลดข้อมูลสาขา...")
            empty_title=t_or("dbpg002.branch.table.empty_title", "ไม่พบข้อมูลสาขา")
            empty_text=t_or("dbpg002.branch.table.empty_text", "ไม่พบสาขาที่ตรงกับเงื่อนไข หรือยังไม่มีสาขาในบริษัทที่เลือก")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg002.branch.search.placeholder", "ค้นหารหัส, ชื่อสาขา, บริษัท...")
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
                        {move || t_or("dbpg002.branch.action.create", "+ เพิ่มสาขาใหม่")}
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
                let (items, first) = paging.slice(ctx.branches.get());
                items.into_iter().enumerate().map(|(idx, b)| {
                let for_edit = b.clone();
                let branch_id = b.id.clone();
                view! {
                    <TableRow class="hover:bg-muted/30 transition-colors">
                        <TableCell class="text-center font-mono text-xs text-muted-foreground">
                            {first + idx as u64}
                        </TableCell>
                        <TableCell class="font-mono font-medium text-xs">
                            <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{b.code.clone()}</span>
                        </TableCell>
                        <TableCell>
                            <div class="text-xs font-semibold text-foreground">
                                {let n = localized(&b.name_th, &b.name_en); if n.is_empty() { "-".to_string() } else { n }}
                            </div>
                        </TableCell>
                        <TableCell>
                            <div class="text-xs space-y-0.5">
                                <div class="text-foreground">{b.company_name.clone()}</div>
                                <div class="text-[11px] font-mono text-muted-foreground">{b.company_code.clone()}</div>
                            </div>
                        </TableCell>
                        <TableCell class="text-xs">
                            {if b.is_headquarter {
                                view! { <Badge variant=BadgeVariant::Default class="text-xs">{b.branch_type_name.clone()}</Badge> }.into_any()
                            } else {
                                view! { <span class="text-muted-foreground">{b.branch_type_name.clone()}</span> }.into_any()
                            }}
                        </TableCell>
                        <TableCell class="text-center text-xs">{b.department_count}</TableCell>
                        <TableCell class="text-center">
                            {if b.active {
                                view! { <Badge variant=BadgeVariant::Success class="text-xs">"Active"</Badge> }
                            } else {
                                view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">"Inactive"</Badge> }
                            }}
                        </TableCell>
                        <TableCell class="text-right">
                            <div class="flex items-center justify-end gap-1.5">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit_branch.run(for_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("common.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                                <PermissionGate program="DBPG002" action=Action::Write>
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Sm
                                        class="h-7 px-2 text-xs"
                                        on_click=Callback::new({
                                            let branch_id = branch_id.clone();
                                            move |_| ctx.open_create_department_in.run(branch_id.clone())
                                        })
                                    >
                                        {t_or("dbpg002.branch.action.add_department", "+ แผนก")}
                                    </Button>
                                </PermissionGate>
                            </div>
                        </TableCell>
                    </TableRow>
                }
            }).collect_view()
            }}
        </DataTable>
    }
}
