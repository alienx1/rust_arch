//! Employee table (shared `DataTable` shell): code, name, department/branch/company, account,
//! status, actions; search, status chips, refresh, add button and paging live in the header/footer.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg003::providers::use_dbpg003;

const USERS_ICON: &str = "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75";

#[component]
pub fn EmployeeTable() -> impl IntoView {
    let ctx = use_dbpg003();
    let can_write = can("DBPG003", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg003.column.code", "รหัสพนักงาน")).class("w-28"),
        DataColumn::new(t_or("dbpg003.column.display_name", "ชื่อ-นามสกุล")).class("min-w-[180px]"),
        DataColumn::new(t_or("dbpg003.column.dept", "แผนก")),
        DataColumn::new(t_or("dbpg003.column.branch_company", "สาขา / บริษัท")),
        DataColumn::new(t_or("dbpg003.column.account", "บัญชีผู้ใช้")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg003.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg003.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg003.table.title", "ทะเบียนพนักงาน (Employee Directory)")
            description=t_or("dbpg003.table.description", "อ้างอิงตาราง: db_employee & db_person_name • ค้นหา/กรอง/แบ่งหน้าที่ฐานข้อมูล")
            icon_path=USERS_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("dbpg003.table.loading", "กำลังโหลดข้อมูลพนักงาน...")
            empty_title=t_or("dbpg003.table.empty_title", "ไม่พบข้อมูลพนักงาน")
            empty_text=t_or("dbpg003.table.empty_text", "ไม่พบพนักงานที่ตรงกับเงื่อนไข หรือยังไม่มีพนักงาน (ต้องมีแผนกใน DBPG002 ก่อนจึงจะเพิ่มพนักงานได้)")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg003.search.placeholder", "ค้นหารหัส, ชื่อ, นามสกุล, ชื่อเล่น...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="DBPG003" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        class="h-9 text-xs shrink-0"
                        on_click=Callback::new(move |_| ctx.open_create.run(()))
                    >
                        {move || t_or("dbpg003.action.create", "+ เพิ่มพนักงาน")}
                    </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("dbpg003.filter.status_label", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("dbpg003.filter.all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("dbpg003.status.working", "ปฏิบัติงาน")),
                        ("INACTIVE".to_string(), t_or("dbpg003.status.left", "พ้นสภาพ")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=ctx.set_status
                />
            }
        >
            {move || {
                let page = ctx.page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, e)| {
                    let for_edit = e.clone();
                    let initial = e.full_name.chars().next().map(|c| c.to_string()).unwrap_or_default();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{e.code.clone()}</span>
                            </TableCell>
                            <TableCell class="font-semibold text-foreground text-xs">
                                <div class="flex items-center gap-2.5">
                                    <div class="h-8 w-8 shrink-0 rounded-full border border-border/60 bg-muted/40 flex items-center justify-center overflow-hidden text-[11px] font-bold text-muted-foreground">
                                        {match e.image_url.clone() {
                                            Some(url) => view! { <img src=url alt="" class="h-full w-full object-cover" loading="lazy" /> }.into_any(),
                                            None => initial.clone().into_any(),
                                        }}
                                    </div>
                                    {e.full_name.clone()}
                                </div>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-medium text-foreground">{e.department_name.clone()}</div>
                                    <div class="text-[11px] font-mono text-muted-foreground">{e.department_code.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="text-foreground">{e.branch_name.clone()}</div>
                                    <div class="text-[11px] text-muted-foreground">{e.company_name.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell class="text-center">
                                {if e.has_user_account {
                                    view! { <Badge variant=BadgeVariant::Default class="text-xs">{t_or("dbpg003.account.has", "มีบัญชี")}</Badge> }.into_any()
                                } else {
                                    view! { <span class="text-xs text-muted-foreground">"-"</span> }.into_any()
                                }}
                            </TableCell>
                            <TableCell class="text-center">
                                {if e.active {
                                    view! { <Badge variant=BadgeVariant::Success class="text-xs">{t_or("dbpg003.status.working", "ปฏิบัติงาน")}</Badge> }
                                } else {
                                    view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">{t_or("dbpg003.status.left", "พ้นสภาพ")}</Badge> }
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
