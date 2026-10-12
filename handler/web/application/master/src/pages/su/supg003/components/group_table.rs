//! Permission group table (shared `DataTable` shell, client-side paging): code, users,
//! readable programs, status, actions (edit / open the matrix).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, ClientPaging, DataColumn, DataTable,
    FilterChips, TableCell, TableRow,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::su::supg003::providers::use_supg003;

const SHIELD_ICON: &str = "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z";

#[component]
pub fn GroupTable() -> impl IntoView {
    let ctx = use_supg003();
    let paging = ClientPaging::new(10);
    let can_write = can("SUPG003", Action::Write);
    let can_read = can("SUPG003", Action::Read);

    let rows = move || {
        let term = ctx.search_query.get().trim().to_lowercase();
        let status = ctx.status_filter.get();
        ctx.groups
            .get()
            .into_iter()
            .filter(|g| match status.as_str() {
                "ACTIVE" => g.active,
                "INACTIVE" => !g.active,
                _ => true,
            })
            .filter(|g| term.is_empty() || g.code.to_lowercase().contains(&term))
            .collect::<Vec<_>>()
    };
    let total = Signal::derive(move || rows().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg003.column.group_code", "รหัสกลุ่มสิทธิ์")).class("w-44"),
        DataColumn::new(t_or("supg003.column.users", "ผู้ใช้")).class("w-24 text-center"),
        DataColumn::new(t_or("supg003.column.programs", "โปรแกรมที่อ่านได้")).class("w-32 text-center"),
        DataColumn::new(t_or("supg003.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("supg003.column.actions", "จัดการ")).class("w-52 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("supg003.table.title", "กลุ่มสิทธิ์ (Permission Groups)")
            description=t_or("supg003.table.description", "อ้างอิงตาราง: su_permission_group & su_permission • เปิดเมทริกซ์เพื่อกำหนดสิทธิ์ READ / WRITE / APPROVE รายโปรแกรม")
            icon_path=SHIELD_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || rows().is_empty())
            loading_text=t_or("supg003.table.loading", "กำลังโหลดกลุ่มสิทธิ์...")
            empty_title=t_or("supg003.table.empty_title", "ไม่พบกลุ่มสิทธิ์")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("supg003.search.placeholder", "ค้นหารหัสกลุ่ม...")
            on_search=Callback::new(move |v| ctx.set_search_query.set(v))
            on_refresh=ctx.refresh
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            actions=move || view! {
                <PermissionGate program="SUPG003" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        class="h-9 text-xs shrink-0"
                        on_click=Callback::new(move |_| ctx.open_create.run(()))
                    >
                        {move || t_or("supg003.action.create", "+ เพิ่มกลุ่มสิทธิ์")}
                    </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <FilterChips
                    label=t_or("supg003.filter.status", "สถานะ:")
                    options=vec![
                        ("ALL".to_string(), t_or("supg003.filter.all", "ทั้งหมด")),
                        ("ACTIVE".to_string(), t_or("common.status.active", "เปิดใช้งาน")),
                        ("INACTIVE".to_string(), t_or("supg003.filter.inactive", "ปิดใช้งาน")),
                    ]
                    value=Signal::derive(move || ctx.status_filter.get())
                    on_change=Callback::new(move |v| ctx.set_status_filter.set(v))
                />
            }
        >
            {move || {
                let (items, first) = paging.slice(rows());
                items.into_iter().enumerate().map(|(idx, g)| {
                    let for_edit = g.clone();
                    let for_matrix = g.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">{first + idx as u64}</TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <div class="flex items-center gap-1.5">
                                    <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{g.code.clone()}</span>
                                    {g.is_protected.then(|| view! { <span title=t_or("supg003.group.protected_tip", "กลุ่มผู้ดูแลระบบ: เข้าถึงได้ทุกโปรแกรม แก้เมทริกซ์/ปิดใช้งานไม่ได้")>"🔒"</span> })}
                                </div>
                            </TableCell>
                            <TableCell class="text-center text-xs">{g.user_count}</TableCell>
                            <TableCell class="text-center text-xs">{g.readable_programs}</TableCell>
                            <TableCell class="text-center">
                                {if g.active {
                                    view! { <Badge variant=BadgeVariant::Success class="text-xs">"Active"</Badge> }
                                } else {
                                    view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">"Inactive"</Badge> }
                                }}
                            </TableCell>
                            <TableCell class="text-right">
                                <div class="flex items-center justify-end gap-1.5">
                                    <Show when=move || can_read.get()>
                                        <Button
                                            variant=ButtonVariant::Outline
                                            size=ButtonSize::Sm
                                            class="h-7 px-2 text-xs"
                                            on_click=Callback::new({
                                                let g = for_matrix.clone();
                                                move |_| ctx.open_matrix.run(g.clone())
                                            })
                                        >
                                            {t_or("supg003.matrix.title", "เมทริกซ์สิทธิ์")}
                                        </Button>
                                    </Show>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        size=ButtonSize::Sm
                                        class="h-7 px-2 text-xs text-primary hover:bg-primary/10"
                                        on_click=Callback::new(move |_| ctx.open_edit.run(for_edit.clone()))
                                    >
                                        {move || if can_write.get() { t_or("common.action.edit", "แก้ไข") } else { t_or("common.action.view", "ดู") }}
                                    </Button>
                                </div>
                            </TableCell>
                        </TableRow>
                    }
                }).collect_view()
            }}
        </DataTable>
    }
}
