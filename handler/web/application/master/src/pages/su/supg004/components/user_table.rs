//! User account table (shared `DataTable` shell): username, employee, department / company,
//! group, status, created, actions (edit / issue a recovery link); server-side paging.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable, FilterChips,
    Select, SelectOption, TableCell, TableRow, data_row_number,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::su::supg004::models::scope_level_label;
use crate::pages::su::supg004::providers::use_supg004;

const USERS_ICON: &str = "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75";

/// `2026-10-09T01:02:03+00:00` → `2026-10-09`
fn date_part(t: &str) -> &str {
    t.split('T').next().unwrap_or(t)
}

#[component]
pub fn UserTable() -> impl IntoView {
    let ctx = use_supg004();
    let can_write = can("SUPG004", Action::Write);

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg004.column.username", "ชื่อผู้ใช้")).class("w-40"),
        DataColumn::new(t_or("supg004.column.employee", "พนักงาน")).class("min-w-[200px]"),
        DataColumn::new(t_or("supg004.column.dept_company", "แผนก / บริษัท")),
        DataColumn::new(t_or("supg004.column.permission_group", "กลุ่มสิทธิ์ / ขอบเขต")).class("w-56"),
        DataColumn::new(t_or("supg004.column.status", "สถานะ")).class("w-28 text-center"),
        DataColumn::new(t_or("supg004.column.created_at", "สร้างเมื่อ")).class("w-28"),
        DataColumn::new(t_or("supg004.column.actions", "จัดการ")).class("w-56 text-right"),
    ];

    view! {
        <DataTable
            title=t_or("supg004.table.title", "บัญชีผู้ใช้ (User Accounts)")
            description=t_or("supg004.table.description", "อ้างอิงตาราง: su_user • บัญชีเข้าสู่ระบบผูกกับพนักงานและกลุ่มสิทธิ์ (ตัวตนและรหัสผ่านอยู่ที่ Ory Kratos)")
            icon_path=USERS_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || ctx.page.get().items.is_empty())
            loading_text=t_or("supg004.table.loading", "กำลังโหลดบัญชีผู้ใช้...")
            empty_title=t_or("supg004.table.empty_title", "ไม่พบบัญชีผู้ใช้")
            empty_text=t_or("supg004.table.empty_text", "ไม่พบบัญชีที่ตรงกับเงื่อนไข")
            search_value=Signal::derive(move || ctx.search_query.get())
            search_placeholder=t_or("supg004.search.placeholder", "ค้นหาชื่อผู้ใช้, รหัสหรือชื่อพนักงาน...")
            on_search=ctx.set_search
            on_refresh=ctx.refresh
            page=Signal::derive(move || ctx.page.get().page)
            limit=Signal::derive(move || ctx.page.get().limit)
            total=Signal::derive(move || ctx.page.get().total)
            on_page=ctx.go_to_page
            on_limit=ctx.set_limit
            actions=move || view! {
                <PermissionGate program="SUPG004" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        class="h-9 text-xs shrink-0"
                        on_click=Callback::new(move |_| ctx.open_create.run(()))
                    >
                        {move || t_or("supg004.action.create", "+ เพิ่มบัญชีผู้ใช้")}
                    </Button>
                </PermissionGate>
            }
            filters=move || view! {
                <div class="flex flex-wrap items-center gap-3">
                    <FilterChips
                        label=t_or("supg004.filter.status", "สถานะ:")
                        options=vec![
                            ("ALL".to_string(), t_or("supg004.filter.all", "ทั้งหมด")),
                            ("ACTIVE".to_string(), t_or("supg004.filter.active", "ใช้งานได้")),
                            ("INACTIVE".to_string(), t_or("supg004.filter.suspended", "ถูกระงับ")),
                        ]
                        value=Signal::derive(move || ctx.status_filter.get())
                        on_change=ctx.set_status
                    />
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-muted-foreground">{t_or("supg004.filter.group", "กลุ่มสิทธิ์:")}</span>
                        <Select
                            value=Signal::derive(move || ctx.group_filter.get())
                            on_change=ctx.set_group
                            class="bg-background text-xs h-8 w-48"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("supg004.filter.all", "ทั้งหมด")}</SelectOption>
                            {move || ctx.groups.get().into_iter().map(|g| {
                                let label = g.display_name();
                                view! { <SelectOption value=g.id.clone()>{label}</SelectOption> }
                            }).collect_view()}
                        </Select>
                    </div>
                </div>
            }
        >
            {move || {
                let page = ctx.page.get();
                let (page_no, limit) = (page.page, page.limit);
                page.items.into_iter().enumerate().map(|(idx, u)| {
                    let for_edit = u.clone();
                    let for_link = u.clone();
                    view! {
                        <TableRow class="hover:bg-muted/30 transition-colors">
                            <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                {data_row_number(page_no, limit, idx)}
                            </TableCell>
                            <TableCell class="font-mono font-medium text-xs">
                                <span class="px-2 py-0.5 rounded bg-muted/60 border border-border/40">{u.username.clone()}</span>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="font-semibold text-foreground">{u.full_name.clone()}</div>
                                    <div class="text-[11px] font-mono text-muted-foreground">{u.employee_code.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell>
                                <div class="text-xs space-y-0.5">
                                    <div class="text-foreground">{u.department_name.clone()}</div>
                                    <div class="text-[11px] text-muted-foreground">{u.company_name.clone()}</div>
                                </div>
                            </TableCell>
                            <TableCell class="font-mono text-xs">
                                {
                                    // Primary assignment first, "+N" for the rest
                                    let more = u.assignments.len().saturating_sub(1);
                                    match u.assignments.first() {
                                        Some(a) => view! {
                                            <div class="space-y-0.5">
                                                <div class="font-medium">{a.group_code.clone()}</div>
                                                <div class="text-[11px] text-muted-foreground font-sans">
                                                    {format!("{}: {}", scope_level_label(&a.scope_level), a.scope_path)}
                                                    {(more > 0).then(|| view! {
                                                        <Badge variant=BadgeVariant::Secondary class="ml-1 text-[10px] px-1.5">{format!("+{more}")}</Badge>
                                                    })}
                                                </div>
                                            </div>
                                        }.into_any(),
                                        None => view! { <span>{u.group_code.clone()}</span> }.into_any(),
                                    }
                                }
                            </TableCell>
                            <TableCell class="text-center">
                                {if u.active {
                                    view! { <Badge variant=BadgeVariant::Success class="text-xs">"Active"</Badge> }
                                } else {
                                    view! { <Badge variant=BadgeVariant::Secondary class="text-xs text-muted-foreground">"Suspended"</Badge> }
                                }}
                            </TableCell>
                            <TableCell class="text-xs text-muted-foreground font-mono">{date_part(&u.created_time).to_string()}</TableCell>
                            <TableCell class="text-right">
                                <div class="flex items-center justify-end gap-1.5">
                                    <Show when=move || can_write.get()>
                                        <Button
                                            variant=ButtonVariant::Outline
                                            size=ButtonSize::Sm
                                            class="h-7 px-2 text-xs"
                                            on_click=Callback::new({
                                                let u = for_link.clone();
                                                move |_| ctx.issue_link.run(u.clone())
                                            })
                                        >
                                            {t_or("supg004.action.recovery_link", "ลิงก์ตั้งรหัสผ่าน")}
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
