//! Program Form (Create / Edit a menu program, SUPG002)
//!
//! System, module and code are chosen on create and fixed afterwards (permissions reference
//! them). Parent folder, route, icon, order, status and names are editable. A program only
//! appears in a menu for groups that hold READ on it (SUPG003); the parent cannot be the program
//! itself or one of its descendants.

use leptos::prelude::*;
use crate::access::{Action, PermissionGate, can};
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Checkbox, DeleteConfirmButton, Input, Select,
    SelectOption, Spinner,
};

use crate::pages::su::supg002::models::ProgramFormInput;
use crate::pages::su::supg002::providers::use_supg002_form;

#[component]
fn Field(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-1.5">
            <label class="text-xs font-semibold text-muted-foreground">{label}</label>
            {children()}
        </div>
    }
}

#[component]
fn ReadOnlyValue(#[prop(into)] value: Signal<String>) -> impl IntoView {
    view! {
        <div class="h-9 px-3 flex items-center rounded-md border border-border/60 bg-muted/30 text-sm font-mono text-muted-foreground">
            {move || value.get()}
        </div>
    }
}

#[component]
pub fn ProgramForm() -> impl IntoView {
    let ctx = use_supg002_form();

    let (system_id, set_system_id) = signal(String::new());
    let (module_id, set_module_id) = signal(String::new());
    let (parent_id, set_parent_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (path, set_path) = signal(String::new());
    let (icon, set_icon) = signal(String::new());
    let (sort_order, set_sort_order) = signal("1".to_string());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = Signal::derive(move || ctx.editing.get().is_some());
    let is_protected = Signal::derive(move || ctx.editing.get().is_some_and(|p| p.is_protected));

    // Seed from the record to edit
    Effect::new(move |_| {
        if let Some(p) = ctx.editing.get() {
            set_system_id.set(p.system_id);
            set_module_id.set(p.module_id);
            set_parent_id.set(p.parent_id.unwrap_or_default());
            set_code.set(p.code);
            set_path.set(p.path.unwrap_or_default());
            set_icon.set(p.icon.unwrap_or_default());
            set_sort_order.set(p.sort_order.to_string());
            set_name_th.set(p.name_th);
            set_name_en.set(p.name_en);
            set_active.set(p.active);
        }
    });

    // Create: preselect system / module from the list filters, else the first of each
    Effect::new(move |_| {
        if is_edit.get() || !system_id.get_untracked().is_empty() {
            return;
        }
        let options = ctx.options.get();
        let system = ctx
            .default_system_id
            .get_untracked()
            .filter(|d| options.systems.iter().any(|s| &s.id == d))
            .or_else(|| options.systems.first().map(|s| s.id.clone()));
        let Some(system) = system else { return };
        let module = ctx
            .default_module_id
            .get_untracked()
            .filter(|d| options.modules.iter().any(|m| &m.id == d && m.system_id == system))
            .or_else(|| options.modules.iter().find(|m| m.system_id == system).map(|m| m.id.clone()));
        set_system_id.set(system);
        set_module_id.set(module.unwrap_or_default());
    });

    let on_system = Callback::new(move |v: String| {
        let first_module = ctx
            .options
            .get()
            .modules
            .into_iter()
            .find(|m| m.system_id == v)
            .map(|m| m.id)
            .unwrap_or_default();
        set_system_id.set(v);
        set_module_id.set(first_module);
        set_parent_id.set(String::new());
    });

    // Parent candidates: same system, never the program itself or anything below it
    let parent_choices = move || {
        let options = ctx.options.get();
        let blocked = ctx
            .editing
            .get()
            .map(|p| options.subtree_of(&p.id))
            .unwrap_or_default();
        let system = system_id.get();
        options
            .programs
            .into_iter()
            .filter(|p| p.system_id == system && !blocked.contains(&p.id))
            .collect::<Vec<_>>()
    };

    let can_write = can("SUPG002", Action::Write);
    let can_approve = can("SUPG002", Action::Approve);

    let on_submit = move |_| {
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();
        let order = sort_order.get().trim().parse::<i32>();

        if !is_edit.get() {
            if system_id.get().is_empty() || module_id.get().is_empty() {
                set_error_msg.set(Some(t_or("supg002.form.error.select_placement", "กรุณาเลือกระบบและโมดูล")));
                return;
            }
            if c.len() < 3 {
                set_error_msg.set(Some(t_or("supg002.form.error.code", "กรุณากรอกรหัสโปรแกรม (3-30 ตัวอักษร/ตัวเลข/_)")));
                return;
            }
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("supg002.form.error.name_th", "กรุณากรอกชื่อเมนูภาษาไทย")));
            return;
        }
        let Ok(order) = order else {
            set_error_msg.set(Some(t_or("supg002.form.error.sort_order", "ลำดับต้องเป็นจำนวนเต็ม")));
            return;
        };
        set_error_msg.set(None);

        ctx.save.run(ProgramFormInput {
            system_id: system_id.get(),
            module_id: module_id.get(),
            parent_id: parent_id.get(),
            code: c,
            path: path.get(),
            icon: icon.get(),
            sort_order: order,
            active: active.get(),
            name_th: th,
            name_en: name_en.get().trim().to_string(),
        });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-5">
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("common.state.loading", "กำลังโหลดข้อมูล...")}</span>
                    </div>
                })}

                {move || error_msg.get().or_else(|| ctx.error.get()).map(|err| view! {
                    <div class="p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20">
                        {err}
                    </div>
                })}

                {move || is_protected.get().then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("supg002.form.protected", "🔒 หน้าจัดการเมนู/สิทธิ์: แก้ไขได้ แต่ซ่อนเมนูไม่ได้ (ไม่เช่นนั้นจะเข้ามาแก้กลับไม่ได้)")}
                    </div>
                })}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("supg002.form.section.placement", "ตำแหน่ง (Placement)")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <Field label=t_or("supg002.form.field.system", "ระบบ (System) *")>
                            {move || if is_edit.get() {
                                let label = ctx.editing.get().map(|p| p.system_code).unwrap_or_default();
                                view! { <ReadOnlyValue value=Signal::derive(move || label.clone()) /> }.into_any()
                            } else {
                                view! {
                                    <Select
                                        value=Signal::derive(move || system_id.get())
                                        on_change=on_system
                                        class="bg-background text-sm"
                                    >
                                        {move || ctx.options.get().systems.into_iter().map(|s| {
                                            let label = s.display_name();
                                            view! { <SelectOption value=s.id.clone()>{label}</SelectOption> }
                                        }).collect_view()}
                                    </Select>
                                }.into_any()
                            }}
                        </Field>
                        <Field label=t_or("supg002.form.field.module", "โมดูล (Module) *")>
                            {move || if is_edit.get() {
                                let label = ctx.editing.get().map(|p| p.module_code).unwrap_or_default();
                                view! { <ReadOnlyValue value=Signal::derive(move || label.clone()) /> }.into_any()
                            } else {
                                view! {
                                    <Select
                                        value=Signal::derive(move || module_id.get())
                                        on_change=Callback::new(move |v| set_module_id.set(v))
                                        class="bg-background text-sm"
                                    >
                                        {move || {
                                            let system = system_id.get();
                                            ctx.options.get().modules.into_iter()
                                                .filter(|m| m.system_id == system)
                                                .map(|m| {
                                                    let label = m.display_name();
                                                    view! { <SelectOption value=m.id.clone()>{label}</SelectOption> }
                                                })
                                                .collect_view()
                                        }}
                                    </Select>
                                }.into_any()
                            }}
                        </Field>
                        <Field label=t_or("supg002.form.field.parent", "โฟลเดอร์แม่ (Parent)")>
                            <Select
                                value=Signal::derive(move || parent_id.get())
                                on_change=Callback::new(move |v| set_parent_id.set(v))
                                class="bg-background text-sm"
                            >
                                <SelectOption value=String::new()>{t_or("supg002.form.parent_none", "— ระดับบนสุด (ไม่มีโฟลเดอร์แม่) —")}</SelectOption>
                                {move || parent_choices().into_iter().map(|p| {
                                    let label = p.display_name();
                                    view! { <SelectOption value=p.id.clone()>{label}</SelectOption> }
                                }).collect_view()}
                            </Select>
                        </Field>
                    </div>
                </div>

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("supg002.form.section.program", "โปรแกรม (Program)")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                        <Field label=t_or("supg002.form.field.code", "รหัสโปรแกรม (Code) *")>
                            {move || if is_edit.get() {
                                view! { <ReadOnlyValue value=Signal::derive(move || code.get()) /> }.into_any()
                            } else {
                                view! {
                                    <Input
                                        value=Signal::derive(move || code.get())
                                        on_input=Callback::new(move |v| set_code.set(v))
                                        placeholder=t_or("supg002.form.placeholder.code", "เช่น DBPG006")
                                        class="font-mono text-sm"
                                    />
                                }.into_any()
                            }}
                        </Field>
                        <Field label=t_or("supg002.form.field.path", "เส้นทาง (Path) — เว้นว่าง = โฟลเดอร์")>
                            <Input
                                value=Signal::derive(move || path.get())
                                on_input=Callback::new(move |v| set_path.set(v))
                                placeholder=t_or("supg002.form.placeholder.path", "เช่น /db/dbpg006")
                                class="font-mono text-sm"
                            />
                        </Field>
                        <Field label=t_or("supg002.form.field.icon", "ไอคอน (Icon)")>
                            <Input
                                value=Signal::derive(move || icon.get())
                                on_input=Callback::new(move |v| set_icon.set(v))
                                placeholder=t_or("supg002.form.placeholder.icon", "เช่น ApartmentOutlined")
                                class="text-sm"
                            />
                        </Field>
                        <Field label=t_or("supg002.form.field.sort_order", "ลำดับ (Sort order)")>
                            <Input
                                value=Signal::derive(move || sort_order.get())
                                on_input=Callback::new(move |v| set_sort_order.set(v))
                                placeholder="1"
                                class="font-mono text-sm"
                            />
                        </Field>
                    </div>
                </div>

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("supg002.form.section.menu_name", "ชื่อเมนู")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <Field label=t_or("supg002.form.field.name_th", "ชื่อเมนู (ภาษาไทย) *")>
                            <Input
                                value=Signal::derive(move || name_th.get())
                                on_input=Callback::new(move |v| set_name_th.set(v))
                                placeholder=t_or("supg002.form.placeholder.name_th", "เช่น ทะเบียนพนักงาน")
                                class="text-sm"
                            />
                        </Field>
                        <Field label=t_or("supg002.form.field.name_en", "ชื่อเมนู (English)")>
                            <Input
                                value=Signal::derive(move || name_en.get())
                                on_input=Callback::new(move |v| set_name_en.set(v))
                                placeholder="e.g. Employees"
                                class="text-sm"
                            />
                        </Field>
                    </div>
                </div>

                <div class="pt-2">
                    {move || view! {
                        <Checkbox
                            checked=Signal::derive(move || active.get())
                            on_change=Callback::new(move |v| set_active.set(v))
                            label=t_or("supg002.form.field.active", "แสดงเมนูในระบบ (Active)")
                            disabled=is_protected.get()
                        />
                    }}
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <Show when=move || is_edit.get() && !is_protected.get() && can_approve.get()>
                        <DeleteConfirmButton
                            label=t_or("supg002.form.delete", "ลบโปรแกรม")
                            confirm_text=t_or("supg002.form.confirm_delete", "ลบโปรแกรมนี้ถาวร?")
                            on_confirm=ctx.delete
                            busy=Signal::derive(move || ctx.is_deleting.get() || ctx.is_saving.get())
                        />
                    </Show>
                    <div class="flex-1" />
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| ctx.cancel.run(()))
                    >
                        {t_or("common.action.cancel", "ยกเลิก")}
                    </Button>
                    <PermissionGate program="SUPG002" action=Action::Write>
                        <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        on_click=Callback::new(on_submit)
                    >
                        {move || if ctx.is_saving.get() { t_or("supg002.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                    </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
