//! Permission matrix (shared `DataTable` shell): programs in tree order with READ / WRITE /
//! APPROVE checkboxes, "select all" per column and per module, a save bar, and filters (module,
//! menu group = folder, free text) that only narrow what is shown: edits and the saved grant set
//! always cover every program.
//!
//! WRITE / APPROVE imply READ; clearing READ clears them. The ADMIN group's matrix is read-only.

use web_package::core::i18n::{localized, t_or, t_fmt};
use leptos::prelude::*;
use web_package::ui::component::{SaveConfirmButton, 
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Checkbox, DataColumn, DataTable,
    TableCell, TableRow,
};

use web_package::ui::component::{Select, SelectOption};

use crate::access::{Action, can};
use crate::pages::su::supg003::models::MatrixRowDto;
use crate::pages::su::supg003::providers::{MatrixColumn, use_supg003_matrix};

const GRID_ICON: &str = "M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z";

#[component]
pub fn MatrixTable() -> impl IntoView {
    let ctx = use_supg003_matrix();
    let can_write = can("SUPG003", Action::Write);
    let read_only = Signal::derive(move || {
        !can_write.get() || ctx.group.get().is_some_and(|g| g.is_protected)
    });

    let columns = vec![
        DataColumn::new(t_or("supg003.matrix.column_module", "ระบบ / โมดูล")).class("w-32"),
        DataColumn::new(t_or("supg003.matrix.column_program", "โปรแกรม")).class("min-w-[260px]"),
        DataColumn::new(t_or("supg003.matrix.column_read", "อ่าน (Read)")).class("w-24 text-center"),
        DataColumn::new(t_or("supg003.matrix.column_write", "เขียน (Write)")).class("w-24 text-center"),
        DataColumn::new(t_or("supg003.matrix.column_approve", "อนุมัติ (Approve)")).class("w-28 text-center"),
    ];

    // Filters (display only)
    let (module_filter, set_module_filter) = signal("ALL".to_string());
    let (group_filter, set_group_filter) = signal("ALL".to_string());
    let (search, set_search) = signal(String::new());

    // `SYSTEM/MODULE` of a row, the value of the module filter
    let module_key = |r: &MatrixRowDto| format!("{}/{}", r.system_code, r.module_code);

    // Is `row` the folder `folder_id` or somewhere below it (parent chain)?
    let is_in_group = |rows: &[MatrixRowDto], row: &MatrixRowDto, folder_id: &str| {
        let mut cursor = Some(row.program_id.clone());
        // The tree is shallow; the bound only guards against a cycle in bad data
        for _ in 0..16 {
            let Some(id) = cursor else { return false };
            if id == folder_id {
                return true;
            }
            cursor = rows.iter().find(|r| r.program_id == id).and_then(|r| r.parent_id.clone());
        }
        false
    };

    // Rows after the filters
    let visible = move || {
        let rows = ctx.rows.get();
        let (module, group) = (module_filter.get(), group_filter.get());
        let term = search.get().trim().to_lowercase();
        rows.iter()
            .filter(|r| module == "ALL" || module_key(r) == module)
            .filter(|r| group == "ALL" || is_in_group(&rows, r, &group))
            .filter(|r| {
                term.is_empty()
                    || r.program_code.to_lowercase().contains(&term)
                    || r.name_th.to_lowercase().contains(&term)
                    || r.name_en.to_lowercase().contains(&term)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    // Folders (menu groups) offered by the group filter, narrowed by the module filter
    let group_options = move || {
        let module = module_filter.get();
        ctx.rows
            .get()
            .into_iter()
            .filter(|r| r.is_folder && (module == "ALL" || module_key(r) == module))
            .map(|r| (r.program_id.clone(), format!("{} — {}", r.program_code, localized(&r.name_th, &r.name_en))))
            .collect::<Vec<_>>()
    };

    // Distinct (system, module) pairs for the quick buttons
    let modules = move || {
        let mut seen: Vec<(String, String)> = Vec::new();
        for r in ctx.rows.get() {
            let key = (r.system_code.clone(), r.module_code.clone());
            if !seen.contains(&key) {
                seen.push(key);
            }
        }
        seen
    };

    view! {
        <DataTable
            title=t_or("supg003.matrix.table_title", "เมทริกซ์สิทธิ์ (Permission Matrix)")
            description=t_or("supg003.matrix.description", "ติ๊กสิทธิ์รายโปรแกรม: READ = เห็นเมนู/อ่านข้อมูล, WRITE = เพิ่มแก้ไข, APPROVE = อนุมัติ • WRITE/APPROVE ต้องมี READ")
            icon_path=GRID_ICON
            columns=columns
            loading=ctx.is_loading
            empty=Signal::derive(move || visible().is_empty())
            loading_text=t_or("supg003.matrix.loading", "กำลังโหลดเมทริกซ์สิทธิ์...")
            empty_title=t_or("supg003.matrix.empty_title", "ไม่พบโปรแกรม")
            empty_text=t_or("supg003.matrix.empty_filtered", "ไม่พบโปรแกรมที่ตรงกับตัวกรอง")
            search_value=Signal::derive(move || search.get())
            search_placeholder=t_or("supg003.matrix.search_placeholder", "ค้นหารหัสหรือชื่อโปรแกรม...")
            on_search=Callback::new(move |v: String| set_search.set(v))
            on_refresh=ctx.reload
            notice=move || view! {
                {move || ctx.notice.get().map(|msg| view! {
                    <div class="p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-lg text-emerald-700 dark:text-emerald-400 text-xs font-medium">
                        {msg}
                    </div>
                })}
                {move || ctx.error.get().map(|err| view! {
                    <div class="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">
                        {err}
                    </div>
                })}
                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}
                {move || (can_write.get() && ctx.group.get().is_some_and(|g| g.is_protected)).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("supg003.matrix.admin_notice", "🔒 กลุ่ม ADMIN เข้าถึงได้ทุกโปรแกรม: ดูได้อย่างเดียว")}
                    </div>
                })}
            }
            filters=move || view! {
                <div class="flex flex-wrap items-center gap-3 pb-1">
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("supg003.matrix.filter_module", "โมดูล:")}</span>
                        <Select
                            value=Signal::derive(move || module_filter.get())
                            on_change=Callback::new(move |v: String| {
                                set_module_filter.set(v);
                                set_group_filter.set("ALL".to_string());
                            })
                            class="bg-background text-xs h-8 w-48"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("supg003.matrix.filter_all", "ทั้งหมด")}</SelectOption>
                            {move || modules().into_iter().map(|(system, module)| {
                                let key = format!("{system}/{module}");
                                view! { <SelectOption value=key.clone()>{key.clone()}</SelectOption> }
                            }).collect_view()}
                        </Select>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("supg003.matrix.filter_group", "กลุ่มเมนู:")}</span>
                        <Select
                            value=Signal::derive(move || group_filter.get())
                            on_change=Callback::new(move |v: String| set_group_filter.set(v))
                            class="bg-background text-xs h-8 w-56"
                        >
                            <SelectOption value="ALL".to_string()>{t_or("supg003.matrix.filter_all", "ทั้งหมด")}</SelectOption>
                            {move || group_options().into_iter().map(|(id, label)| view! {
                                <SelectOption value=id.clone()>{label}</SelectOption>
                            }).collect_view()}
                        </Select>
                    </div>
                </div>
                <div class="flex flex-wrap items-center gap-2 pt-2">
                    <span class="text-xs text-muted-foreground whitespace-nowrap">{t_or("supg003.matrix.select_all", "เลือกทั้งหมด:")}</span>
                    {[(t_or("supg003.matrix.read", "อ่าน"), MatrixColumn::Read), (t_or("supg003.matrix.write", "เขียน"), MatrixColumn::Write), (t_or("supg003.matrix.approve", "อนุมัติ"), MatrixColumn::Approve)]
                        .into_iter()
                        .map(|(label, column)| view! {
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                class="h-7 px-2.5 text-xs"
                                disabled=read_only
                                on_click=Callback::new(move |_| ctx.set_column.run((column, true)))
                            >
                                {t_fmt("supg003.matrix.column_all_programs", "{label} ทุกโปรแกรม", &[("label", &label.to_string())])}
                            </Button>
                        })
                        .collect_view()}
                    {move || modules().into_iter().map(|(system, module)| {
                        let (s1, m1) = (system.clone(), module.clone());
                        let (s2, m2) = (system.clone(), module.clone());
                        view! {
                            <div class="flex items-center rounded-md border border-border/60 text-xs">
                                <span class="px-2 font-mono text-muted-foreground">{format!("{system}/{module}")}</span>
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs"
                                    disabled=read_only
                                    on_click=Callback::new(move |_| ctx.set_module.run((s1.clone(), m1.clone(), true)))
                                >
                                    {t_or("supg003.filter.all", "ทั้งหมด")}
                                </Button>
                                <Button
                                    variant=ButtonVariant::Ghost
                                    size=ButtonSize::Sm
                                    class="h-7 px-2 text-xs text-muted-foreground"
                                    disabled=read_only
                                    on_click=Callback::new(move |_| ctx.set_module.run((s2.clone(), m2.clone(), false)))
                                >
                                    {t_or("supg003.matrix.clear", "ล้าง")}
                                </Button>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }
            actions=move || view! {
                <Show when=move || can_write.get()>
                <SaveConfirmButton
                    label=Signal::derive(move || if ctx.is_saving.get() { t_or("supg003.form.saving", "กำลังบันทึก...") } else if ctx.is_dirty.get() { t_or("supg003.matrix.save_dirty", "💾 บันทึกสิทธิ์") } else { t_or("supg003.matrix.save", "บันทึกสิทธิ์") })
                    on_confirm=Callback::new(move |_| ctx.save.run(()))
                    busy=Signal::derive(move || ctx.is_saving.get())
                    disabled=Signal::derive(move || read_only.get() || !ctx.is_dirty.get())
                />
                </Show>
            }
        >
            {move || visible().into_iter().map(|r| {
                let id_read = r.program_id.clone();
                let id_write = r.program_id.clone();
                let id_approve = r.program_id.clone();
                let indent = format!("padding-left: {}rem", r.depth as f32 * 1.25);
                let (read, write, approve) = (r.read, r.write, r.approve);
                view! {
                    <TableRow class="hover:bg-muted/30 transition-colors">
                        <TableCell class="font-mono text-[11px] text-muted-foreground">
                            {format!("{}/{}", r.system_code, r.module_code)}
                        </TableCell>
                        <TableCell>
                            <div class="text-xs flex items-center gap-1.5" style=indent>
                                {if r.is_folder { "📁" } else { "📄" }}
                                <span class="font-mono font-medium text-foreground">{r.program_code.clone()}</span>
                                <span class="text-muted-foreground">{localized(&r.name_th, &r.name_en)}</span>
                                {(!r.program_active).then(|| view! {
                                    <Badge variant=BadgeVariant::Secondary class="text-[10px]">{t_or("supg003.matrix.menu_hidden", "เมนูซ่อนอยู่")}</Badge>
                                })}
                            </div>
                        </TableCell>
                        <TableCell class="text-center">
                            <div class="flex justify-center">
                                <Checkbox
                                    checked=Signal::derive(move || read)
                                    on_change=Callback::new(move |v| ctx.set_cell.run((id_read.clone(), MatrixColumn::Read, v)))
                                    disabled=read_only.get()
                                />
                            </div>
                        </TableCell>
                        <TableCell class="text-center">
                            <div class="flex justify-center">
                                <Checkbox
                                    checked=Signal::derive(move || write)
                                    on_change=Callback::new(move |v| ctx.set_cell.run((id_write.clone(), MatrixColumn::Write, v)))
                                    disabled=read_only.get()
                                />
                            </div>
                        </TableCell>
                        <TableCell class="text-center">
                            <div class="flex justify-center">
                                <Checkbox
                                    checked=Signal::derive(move || approve)
                                    on_change=Callback::new(move |v| ctx.set_cell.run((id_approve.clone(), MatrixColumn::Approve, v)))
                                    disabled=read_only.get()
                                />
                            </div>
                        </TableCell>
                    </TableRow>
                }
            }).collect_view()}
        </DataTable>
    }
}
