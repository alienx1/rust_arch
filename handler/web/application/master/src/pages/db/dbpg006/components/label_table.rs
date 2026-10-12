//! Label table (shared `DataTable` shell): category, key, one line per language, actions; search,
//! category filter, refresh, add button and server-side paging live in the header/footer.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, Select,
    SelectOption, TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg006::providers::use_dbpg006;

const LANGUAGE_ICON: &str = "M5 8l6 6M4 14l6-6 2-3M2 5h12M7 2h1M22 22l-5-10-5 10M14 18h6";

#[component]
pub fn LabelTable() -> impl IntoView {
    let ctx = use_dbpg006();
    let can_write = can("DBPG006", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("dbpg006.column.category", "หมวด (Category)")).class("w-40"),
        DataColumn::new(t_or("dbpg006.column.key", "คีย์ (Key)")).class("min-w-[200px]"),
        DataColumn::new(t_or("dbpg006.column.text", "ข้อความ (Text)")),
        DataColumn::new(t_or("dbpg006.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("dbpg006.table.title", "ตารางข้อความแปลภาษา (i18n Labels)")
            description=t_or("dbpg006.table.description", "อ้างอิงตาราง: db_label • 1 รายการ = หมวด + คีย์ (มีข้อความครบทุกภาษา) • ค้นหา/กรอง/แบ่งหน้าที่ฐานข้อมูล")
            icon_path=LANGUAGE_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("dbpg006.state.loading", "กำลังโหลดข้อความแปลภาษา...")
            empty_title=t_or("dbpg006.state.empty_title", "ไม่พบข้อความแปลภาษา")
            empty_text=t_or("dbpg006.state.empty_text", "ไม่พบรายการที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("dbpg006.search.placeholder", "ค้นหาหมวด คีย์ หรือข้อความ...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="DBPG006" action=Action::Write>
                <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {move || t_or("dbpg006.action.create", "+ เพิ่มข้อความ")}
                </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <div class="flex items-center gap-2 pt-2 max-w-md">
                    <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("dbpg006.filter.category", "หมวด:")}</span>
                    <Select
                        value=Signal::derive(move || ctx.category_filter.get())
                        on_change=ctx.set_category
                        class="bg-background text-xs h-8"
                    >
                        <SelectOption value="ALL".to_string()>{t_or("dbpg006.filter.all_categories", "ทุกหมวด (All Categories)")}</SelectOption>
                        {move || ctx.categories.get().into_iter().map(|c| {
                            let label = t_fmt("dbpg006.filter.category_option", "{category} • {count} รายการ", &[("category", c.category.as_str()), ("count", &c.entries.to_string())]);
                            view! { <SelectOption value=c.category.clone()>{label}</SelectOption> }
                        }).collect_view()}
                    </Select>
                </div>
            }
        >
            {move || {
                let page = ctx.page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, e)| {
                    let for_edit = e.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell>
                                <Badge variant=BadgeVariant::Outline class="text-xs font-mono">
                                    {e.category.clone()}
                                </Badge>
                            </TableCell>
                            <TableCell class="font-mono text-xs break-all">{e.key.clone()}</TableCell>
                            <TableCell class="text-xs">
                                <div class="space-y-1">
                                    {e.values.into_iter().map(|v| view! {
                                        <div class="flex items-center gap-2 max-w-xl">
                                            <span class="px-1.5 py-0.5 rounded bg-muted/60 border border-border/40 font-mono uppercase text-[10px] text-muted-foreground shrink-0">
                                                {v.language_code.clone()}
                                            </span>
                                            <span class="truncate" title=v.value.clone()>{v.value.clone()}</span>
                                        </div>
                                    }).collect_view()}
                                </div>
                            </TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit.run(for_edit.clone()))
                                >
                                    {move || if can_write.get() { t_or("dbpg006.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                </Button>
                            </TableCell>
                        </TableRow>
                    }
                }).collect_view()
            }}
        </DataTable>
    }
}
