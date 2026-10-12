//! Systems / modules tables (shared `DataTable` shell, client-side paging)

use leptos::prelude::*;
use crate::access::{Action, PermissionGate, can};
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, ClientPaging, DataColumn, DataTable,
    FilterChips, TableCell, TableRow,
};

use crate::pages::su::supg001::providers::use_supg001;

const LAYERS_ICON: &str = "M12 2 2 7l10 5 10-5zM2 17l10 5 10-5M2 12l10 5 10-5";

fn status_chips(
    value: Signal<String>,
    on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <FilterChips
            label=t_or("supg001.filter.status", "สถานะ:")
            options=vec![
                ("ALL".to_string(), t_or("supg001.filter.all", "ทั้งหมด").to_string()),
                ("ACTIVE".to_string(), t_or("common.status.active", "เปิดใช้งาน").to_string()),
                ("INACTIVE".to_string(), t_or("supg001.filter.inactive", "ปิดใช้งาน").to_string()),
            ]
            value=value
            on_change=on_change
        />
    }
}

fn status_badge(active: bool) -> impl IntoView {
    if active {
        view! { <Badge variant=BadgeVariant::Success class="text-xs">"Active"</Badge> }
    } else {
        view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">"Inactive"</Badge> }
    }
}

/// `true` when `status` ("ALL" / "ACTIVE" / "INACTIVE") admits a row with this `active` flag
fn admits(status: &str, active: bool) -> bool {
    match status {
        "ACTIVE" => active,
        "INACTIVE" => !active,
        _ => true,
    }
}

fn contains(term: &str, fields: &[&str]) -> bool {
    term.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(term))
}

#[component]
pub fn SystemsTable() -> impl IntoView {
    let ctx = use_supg001();
    let can_write = can("SUPG001", Action::Write);
    let paging = ClientPaging::new(10);

    let rows = move || {
        let term = ctx.search_query.get().trim().to_lowercase();
        let status = ctx.status_filter.get();
        ctx.systems
            .get()
            .into_iter()
            .filter(|s| admits(&status, s.active) && contains(&term, &[&s.code, &s.name_th, &s.name_en]))
            .collect::<Vec<_>>()
    };
    let total = Signal::derive(move || rows().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg001.column.code", "รหัสระบบ")).class("w-28"),
        DataColumn::new(t_or("supg001.column.name_one", "ชื่อระบบ")).class("min-w-[200px]"),
        DataColumn::new(t_or("supg001.column.modules", "โมดูล")).class("w-24 text-center"),
        DataColumn::new(t_or("supg001.column.programs", "โปรแกรม")).class("w-24 text-center"),
        DataColumn::new(t_or("supg001.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("supg001.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("supg001.table.systems.title", "ตารางระบบ (Systems)")
            description=t_or("supg001.table.systems.description", "อ้างอิงตาราง: db_system & db_system_label • ปิดระบบ = เมนูของโปรแกรมทั้งหมดในระบบนั้นหายไป")
            icon_path=LAYERS_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || rows().is_empty())
            loading_text=t_or("supg001.table.systems.loading", "กำลังโหลดข้อมูลระบบ...")
            empty_title=t_or("supg001.table.systems.empty", "ไม่พบข้อมูลระบบ")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("supg001.search.placeholder", "ค้นหารหัส หรือชื่อระบบ...")
            on_search=Callback::new(move |v| ctx.set_search_query.set(v))
            on_refresh=ctx.refresh
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            actions=move || view! {
                <PermissionGate program="SUPG001" action=Action::Write>
                    <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {move || t_or("supg001.action.create_system", "+ เพิ่มระบบ")}
                </Button>
                </PermissionGate>
            }
            filters=move || status_chips(
                Signal::derive(move || ctx.status_filter.get()),
                Callback::new(move |v| ctx.set_status_filter.set(v)),
            )
        >
            {move || {
                let (items, first) = paging.slice(rows());
                items.into_iter().enumerate().map(|(idx, s)| {
                    let for_edit = s.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">{first + idx as u64}</TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{s.code.clone()}</span>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-semibold text-foreground flex items-center gap-1.5">
                                        {web_package::core::i18n::localized(&s.name_th, &s.name_en)}
                                        {s.is_protected.then(|| view! { <span title=t_or("supg001.table.systems.protected", "ระบบหลักของการจัดการสิทธิ์: ปิดใช้งานไม่ได้")>"🔒"</span> })}
                                    </div>
                                </div>
                            </TableCell>
                            <TableCell class="text-center text-xs">{s.module_count}</TableCell>
                            <TableCell class="text-center text-xs">{s.program_count}</TableCell>
                            <TableCell class="text-center">{status_badge(s.active)}</TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit_system.run(for_edit.clone()))
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

#[component]
pub fn ModulesTable() -> impl IntoView {
    let ctx = use_supg001();
    let can_write = can("SUPG001", Action::Write);
    let paging = ClientPaging::new(10);

    let rows = move || {
        let term = ctx.search_query.get().trim().to_lowercase();
        let status = ctx.status_filter.get();
        ctx.modules
            .get()
            .into_iter()
            .filter(|m| {
                admits(&status, m.active)
                    && contains(&term, &[&m.code, &m.name_th, &m.name_en, &m.system_code])
            })
            .collect::<Vec<_>>()
    };
    let total = Signal::derive(move || rows().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg001.module.column.code", "รหัสโมดูล")).class("w-28"),
        DataColumn::new(t_or("supg001.module.column.name_one", "ชื่อโมดูล")).class("min-w-[200px]"),
        DataColumn::new(t_or("supg001.module.column.system", "ระบบ")),
        DataColumn::new(t_or("supg001.column.programs", "โปรแกรม")).class("w-24 text-center"),
        DataColumn::new(t_or("supg001.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("supg001.column.actions", "จัดการ")).class("w-24 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("supg001.table.modules.title", "ตารางโมดูล (Modules)")
            description=t_or("supg001.table.modules.description", "อ้างอิงตาราง: db_module & db_module_label • ปิดโมดูล = เมนูของโปรแกรมในโมดูลนั้นหายไป")
            icon_path=LAYERS_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || rows().is_empty())
            loading_text=t_or("supg001.table.modules.loading", "กำลังโหลดข้อมูลโมดูล...")
            empty_title=t_or("supg001.table.modules.empty", "ไม่พบข้อมูลโมดูล")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("supg001.search.placeholder_module", "ค้นหารหัส หรือชื่อโมดูล...")
            on_search=Callback::new(move |v| ctx.set_search_query.set(v))
            on_refresh=ctx.refresh
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            actions=move || view! {
                <PermissionGate program="SUPG001" action=Action::Write>
                    <Button
                    variant=ButtonVariant::Default
                    size=ButtonSize::Sm
                    class="h-9 text-xs shrink-0"
                    on_click=Callback::new(move |_| ctx.open_create.run(()))
                >
                    {move || t_or("supg001.action.create_module", "+ เพิ่มโมดูล")}
                </Button>
                </PermissionGate>
            }
            filters=move || status_chips(
                Signal::derive(move || ctx.status_filter.get()),
                Callback::new(move |v| ctx.set_status_filter.set(v)),
            )
        >
            {move || {
                let (items, first) = paging.slice(rows());
                items.into_iter().enumerate().map(|(idx, m)| {
                    let for_edit = m.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">{first + idx as u64}</TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{m.code.clone()}</span>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-semibold text-foreground flex items-center gap-1.5">
                                        {web_package::core::i18n::localized(&m.name_th, &m.name_en)}
                                        {m.is_protected.then(|| view! { <span title=t_or("supg001.table.modules.protected", "โมดูลหลักของการจัดการสิทธิ์: ปิดใช้งานไม่ได้")>"🔒"</span> })}
                                    </div>
                                </div>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="text-foreground">{m.system_name.clone()}</div>
                                    <div class="text-[11px] font-mono text-muted-foreground">{m.system_code.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell class="text-center text-xs">{m.program_count}</TableCell>
                            <TableCell class="text-center">{status_badge(m.active)}</TableCell>
                            <TableCell class="text-right">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                    on_click=Callback::new(move |_| ctx.open_edit_module.run(for_edit.clone()))
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
