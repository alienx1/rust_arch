//! Effective Permissions Matrix Table Component (shared `DataTable` shell)

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{
    Badge, BadgeVariant, ClientPaging, DataColumn, DataTable, FilterChips, TableCell, TableRow,
};

use crate::pages::su::supg005::models::ProgramPermissionDto;
use crate::pages::su::supg005::providers::use_supg005;

#[component]
pub fn SessionPermissionsTable(permissions: Vec<ProgramPermissionDto>) -> impl IntoView {
    let ctx = use_supg005();
    let permissions_list = StoredValue::new(permissions);

    let on_search = move |val: String| {
        ctx.set_search_query.set(val);
    };

    let filtered_permissions = move || {
        let q = ctx.search_query.get().trim().to_lowercase();
        let mod_filter = ctx.module_filter.get();

        permissions_list
            .get_value()
            .into_iter()
            .filter(|p| {
                let matches_module = mod_filter == "ALL" || p.module_code == mod_filter;
                let matches_query = q.is_empty()
                    || p.program_code.to_lowercase().contains(&q)
                    || p.program_name.to_lowercase().contains(&q)
                    || p.module_name.to_lowercase().contains(&q);
                matches_module && matches_query
            })
            .collect::<Vec<_>>()
    };

    let paging = ClientPaging::new(10);
    let total = Signal::derive(move || filtered_permissions().len() as u64);

    let columns = vec![
        DataColumn::new(t_or("supg005.perm.column_code", "รหัสโปรแกรม")).class("w-28"),
        DataColumn::new(t_or("supg005.perm.column_name", "ชื่อโปรแกรม / เมนู")),
        DataColumn::new(t_or("supg005.perm.column_system", "หมวดหมู่ระบบ")).class("w-36"),
        DataColumn::new(t_or("supg005.perm.column_read", "อ่าน (Read)")).class("w-20 text-center"),
        DataColumn::new(t_or("supg005.perm.column_write", "เขียน (Write)")).class("w-20 text-center"),
        DataColumn::new(t_or("supg005.perm.column_approve", "อนุมัติ (Approve)")).class("w-24 text-center"),
    ];

    view! {
        <DataTable
            title=t_or("supg005.perm.title", "เมทริกซ์สิทธิ์การใช้งาน (Effective Permissions Matrix)")
            description=t_or("supg005.perm.description", "สิทธิ์การเข้าถึงเมนูและโปรแกรมที่คำนวณจากกลุ่มสิทธิ์ (su_permission_group) และขอบเขตองค์กรปัจจุบัน")
            icon_path="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
            columns=columns
            empty=Signal::derive(move || filtered_permissions().is_empty())
            empty_title=t_or("supg005.perm.empty_title", "ไม่พบสิทธิ์โปรแกรม")
            empty_text=t_or("supg005.perm.empty_text", "ไม่พบสิทธิ์โปรแกรมที่ตรงกับเงื่อนไขการค้นหา")
            page=Signal::derive(move || paging.page_for(total.get()))
            limit=paging.size_signal()
            on_limit=paging.on_limit()
            total=total
            on_page=Callback::new(move |p| paging.set_page(p))
            search_placeholder=t_or("supg005.perm.search_placeholder", "ค้นหารหัสโปรแกรม หรือชื่อเมนู...")
            on_search=Callback::new(on_search)
            filters=move || view! {
                <FilterChips
                    label=t_or("supg005.perm.filter_system", "กรองตามระบบ:")
                    options=vec![
                        ("ALL".to_string(), t_or("supg005.perm.all", "ทั้งหมด")),
                        ("SU".to_string(), "SU (Security)".to_string()),
                        ("DB".to_string(), "DB (Master)".to_string()),
                        ("SY".to_string(), "SY (System)".to_string()),
                        ("AL".to_string(), "AL (Audit)".to_string()),
                    ]
                    value=Signal::derive(move || ctx.module_filter.get())
                    on_change=Callback::new(move |m| ctx.set_module_filter.set(m))
                />
            }
        >
            {move || paging.slice(filtered_permissions()).0.into_iter().map(|p| {
                view! {
                    <TableRow class="hover:bg-muted/40 transition-colors">
                        <TableCell class="font-mono font-medium text-xs text-foreground">
                            {p.program_code.clone()}
                        </TableCell>
                        <TableCell class="font-medium text-xs text-foreground">
                            {p.program_name.clone()}
                        </TableCell>
                        <TableCell>
                            <Badge variant=BadgeVariant::Secondary class="text-[11px] font-sans">
                                {p.module_name.clone()}
                            </Badge>
                        </TableCell>
                        <TableCell class="text-center">
                            {if p.can_read {
                                view! {
                                    <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 text-xs font-bold">
                                        "✓"
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span class="text-muted-foreground/40 text-xs">"-"</span> }.into_any()
                            }}
                        </TableCell>
                        <TableCell class="text-center">
                            {if p.can_write {
                                view! {
                                    <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-blue-500/15 text-blue-600 dark:text-blue-400 text-xs font-bold">
                                        "✓"
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span class="text-muted-foreground/40 text-xs">"-"</span> }.into_any()
                            }}
                        </TableCell>
                        <TableCell class="text-center">
                            {if p.can_approve {
                                view! {
                                    <span class="inline-flex items-center justify-center w-5 h-5 rounded-full bg-purple-500/15 text-purple-600 dark:text-purple-400 text-xs font-bold">
                                        "✓"
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span class="text-muted-foreground/40 text-xs">"-"</span> }.into_any()
                            }}
                        </TableCell>
                    </TableRow>
                }
            }).collect_view()}
        </DataTable>
    }
}
