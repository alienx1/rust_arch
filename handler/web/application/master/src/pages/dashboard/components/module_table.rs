//! Master Modules Table Component (shared `DataTable` shell)

use crate::pages::dashboard::providers::use_dashboard;
use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Badge, BadgeVariant, ClientPaging, DataColumn, DataTable, TableCell, TableRow,
};

const MODULE_ICON: &str = "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96 12 12.01l8.73-5.05M12 22.08V12";

#[component]
pub fn MasterModuleTable() -> impl IntoView {
    let ctx = use_dashboard();

    let paging = ClientPaging::new(5);
    let total = Signal::derive(move || ctx.data.get().map_or(0, |d| d.modules.len() as u64));

    let columns = vec![
        DataColumn::new(t_or("sypg000.column.module_code", "รหัสโมดูล")),
        DataColumn::new(t_or("sypg000.column.module_name", "ชื่อโมดูล (TH / EN)")),
        DataColumn::new(t_or("sypg000.column.system", "ระบบงาน")),
        DataColumn::new(t_or("sypg000.column.status", "สถานะ")),
    ];

    view! {
        <DataTable
            title=t_or("sypg000.table.modules.title", "รายการโมดูลข้อมูลหลัก (Registered Modules)")
            description=t_or("sypg000.table.modules.description", "โมดูลที่ลงทะเบียนในฐานข้อมูล (db_module) และสถานะการทำงานจริง")
            icon_path=MODULE_ICON
            columns=columns
            loading=Signal::derive(move || ctx.is_loading.get() && ctx.data.get().is_none())
            empty=Signal::derive(move || ctx.data.get().is_none_or(|d| d.modules.is_empty()))
            loading_text=t_or("sypg000.table.modules.loading", "กำลังโหลดรายการโมดูลจากฐานข้อมูล...")
            empty_title=t_or("sypg000.table.modules.empty", "ไม่พบข้อมูลโมดูล")
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            actions=move || view! {
                <Badge variant=BadgeVariant::Outline>
                    {move || {
                        ctx.data.get()
                            .map(|d| t_fmt("sypg000.table.modules.count", "{count} โมดูล", &[("count", &d.modules.len().to_string())]))
                            .unwrap_or_else(|| t_fmt("sypg000.table.modules.count", "{count} โมดูล", &[("count", "4")]))
                    }}
                </Badge>
            }
        >
            {move || {
                ctx.data.get().map(|data| {
                    paging.slice(data.modules).0.into_iter().map(|m| {
                        view! {
                            <TableRow class="hover:bg-muted/30 transition-colors">
                                <TableCell class="font-mono text-xs font-semibold text-primary">{m.code}</TableCell>
                                <TableCell>
                                    <div class="text-xs font-semibold text-foreground">
                                        {web_package::core::i18n::localized(&m.label_th, &m.label_en)}
                                    </div>
                                </TableCell>
                                <TableCell>
                                    <Badge variant=BadgeVariant::Secondary class="text-[10px] font-mono">
                                        {m.system_code}
                                    </Badge>
                                </TableCell>
                                <TableCell>
                                    {if m.active {
                                        view! { <Badge variant=BadgeVariant::Success class="text-xs">{t_or("sypg000.status.ready", "พร้อมใช้งาน")}</Badge> }
                                    } else {
                                        view! { <Badge variant=BadgeVariant::Warning class="text-xs">{t_or("sypg000.status.disabled", "ปิดใช้งาน")}</Badge> }
                                    }}
                                </TableCell>
                            </TableRow>
                        }
                    }).collect_view()
                })
            }}
        </DataTable>
    }
}
