//! Company table (shared `DataTable` shell): code, names, parent organization, type, status,
//! actions; search, status chips and refresh live in the header.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, ClientPaging, FilterChips,
    TableCell, TableRow,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg000::models::CompanyDto;
use crate::pages::db::dbpg000::providers::use_dbpg000;
use crate::routes::DBPG002;

const COMPANY_ICON: &str = "M2 7h20v14H2zM16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16";

#[component]
pub fn CompanyDataTable() -> impl IntoView {
    let ctx = use_dbpg000();
    let can_write = can("DBPG001", Action::Write);

    let paging = ClientPaging::new(10);
    let total = Signal::derive(move || ctx.companies.get().len() as u64);

    let on_edit = move |comp: CompanyDto| ctx.open_edit_company.run(comp);

    let columns = vec![
        DataColumn::new(t_or("dbpg000.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg000.column.code", "รหัสบริษัท")).class("w-28"),
        DataColumn::new(t_or("dbpg000.column.name", "ชื่อบริษัท")).class("min-w-[200px]"),
        DataColumn::new(t_or("dbpg000.column.org", "สังกัดองค์กร")),
        DataColumn::new(t_or("dbpg000.column.type", "ประเภท")),
        DataColumn::new(t_or("dbpg000.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("dbpg000.column.actions", "จัดการ")).class("w-36 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg000.table.title", "ตารางรายชื่อบริษัทในเครือ (Company Directory)")
            description=t_or("dbpg000.table.description", "อ้างอิงตาราง: db_company & db_company_label")
            icon_path=COMPANY_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.companies.get().is_empty())
            loading_text=t_or("dbpg000.table.loading", "กำลังโหลดข้อมูลบริษัท...")
            empty_title=t_or("dbpg000.table.empty_title", "ไม่พบข้อมูลบริษัท")
            empty_text=t_or("dbpg000.table.empty_text", "ไม่พบบริษัทที่ตรงกับเงื่อนไขการค้นหา หรือยังไม่มีบริษัทในกลุ่มองค์กรที่เลือก")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg000.search.placeholder", "ค้นหาชื่อบริษัท, รหัส, องค์กรแม่...")
            on_search=Callback::new(move |v| ctx.set_search_query.set(v))
            on_refresh=ctx.refresh
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            empty_action=move || view! {
                <PermissionGate program="DBPG001" action=Action::Write>
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    class="text-xs"
                    on_click=Callback::new(move |_| ctx.open_create_company.run(()))
                >
                    {t_or("dbpg000.action.create_company", "+ เพิ่มบริษัทใหม่")}
                </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("dbpg000.filter.status", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("dbpg000.filter.all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("common.status.active", "เปิดใช้งาน")),
                        ("INACTIVE".to_string(), t_or("dbpg000.status.inactive", "ปิดใช้งาน")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=Callback::new(move |v| ctx.set_status_filter.set(v))
                />
            }
        >
            {move || {
                let (items, first) = paging.slice(ctx.companies.get());
                items.into_iter().enumerate().map(|(idx, comp)| {
                let comp_edit = comp.clone();
                let comp_initial = comp.code.chars().next().map(|c| c.to_string()).unwrap_or_default();
                let branch_url = format!("{DBPG002}?company_id={}", comp.id);
                view! {
                    <TableRow class="hover:bg-muted/30 transition-colors">
                        <TableCell class="text-center font-mono text-xs text-muted-foreground">
                            {first + idx as u64}
                        </TableCell>
                        <TableCell class="font-mono font-medium text-xs">
                            <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{comp.code.clone()}</span>
                        </TableCell>
                        <TableCell>
                            <div class="flex items-center gap-2.5">
                                <div class="h-8 w-8 shrink-0 rounded-md border border-border/60 bg-muted/40 flex items-center justify-center overflow-hidden text-[11px] font-bold text-muted-foreground">
                                    {match comp.image_url.clone() {
                                        Some(url) => view! { <img src=url alt="" class="h-full w-full object-contain" loading="lazy" /> }.into_any(),
                                        None => comp_initial.clone().into_any(),
                                    }}
                                </div>
                                <div class="text-xs font-semibold text-foreground">
                                    {
                                        let name = localized(&comp.name_th, &comp.name_en);
                                        if name.is_empty() { "-".to_string() } else { name }
                                    }
                                </div>
                            </div>
                        </TableCell>
                        <TableCell>
                            <div class="flex items-center gap-1.5">
                                <Badge variant=BadgeVariant::Outline class="text-xs py-0.5 border-primary/30 text-primary">
                                    {comp.organization_code.clone()}
                                </Badge>
                                <span class="text-xs text-muted-foreground truncate max-w-[140px]" title=comp.organization_name.clone()>
                                    {comp.organization_name.clone()}
                                </span>
                            </div>
                        </TableCell>
                        <TableCell class="text-xs text-muted-foreground">{comp.company_type_name.clone()}</TableCell>
                        <TableCell class="text-center">
                            {if comp.active {
                                view! { <Badge variant=BadgeVariant::Success class="text-xs">{t_or("common.status.active", "Active")}</Badge> }
                            } else {
                                view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">{t_or("common.status.inactive", "Inactive")}</Badge> }
                            }}
                        </TableCell>
                        <TableCell class="text-right">
                            <div class="flex items-center justify-end gap-1.5">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| on_edit(comp_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("common.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                                <a
                                    href=branch_url
                                    class="inline-flex items-center justify-center rounded-md text-xs h-7 px-2 border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
                                    title=t_or("dbpg000.action.goto_branches", "ไปยังระบบกำหนดสาขา / แผนก (DBPG002)")
                                >
                                    {t_or("sypg000.kpi.branches", "สาขา")}
                                </a>
                            </div>
                        </TableCell>
                    </TableRow>
                }
            }).collect_view()
            }}
        </DataTable>
    }
}
