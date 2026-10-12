//! System / module form (SUPG001)
//!
//! Code (and a module's system) are chosen on create and fixed afterwards: permissions, routes
//! and the menu reference them. Protected units cannot be switched off.

use leptos::prelude::*;
use crate::access::{Action, PermissionGate, can};
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Checkbox, DeleteConfirmButton, Input, Select,
    SelectOption, Spinner,
};

use crate::pages::su::supg001::models::{UnitFormInput, UnitKind};
use crate::pages::su::supg001::providers::use_supg001_form;

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
pub fn UnitForm() -> impl IntoView {
    let ctx = use_supg001_form();
    let kind = ctx.kind;
    let can_write = can("SUPG001", Action::Write);
    let can_approve = can("SUPG001", Action::Approve);

    let (system_id, set_system_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = Signal::derive(move || {
        ctx.editing_system.get().is_some() || ctx.editing_module.get().is_some()
    });
    let is_protected = Signal::derive(move || {
        ctx.editing_system.get().is_some_and(|s| s.is_protected)
            || ctx.editing_module.get().is_some_and(|m| m.is_protected)
    });

    Effect::new(move |_| {
        if let Some(s) = ctx.editing_system.get() {
            set_code.set(s.code);
            set_name_th.set(s.name_th);
            set_name_en.set(s.name_en);
            set_active.set(s.active);
        }
        if let Some(m) = ctx.editing_module.get() {
            set_system_id.set(m.system_id);
            set_code.set(m.code);
            set_name_th.set(m.name_th);
            set_name_en.set(m.name_en);
            set_active.set(m.active);
        }
    });

    // Module create: preselect the system from the list filter, else the first one
    Effect::new(move |_| {
        if kind != UnitKind::Module || is_edit.get() || !system_id.get_untracked().is_empty() {
            return;
        }
        let systems = ctx.systems.get();
        let chosen = ctx
            .default_system_id
            .get_untracked()
            .filter(|d| systems.iter().any(|s| &s.id == d))
            .or_else(|| systems.first().map(|s| s.id.clone()));
        if let Some(id) = chosen {
            set_system_id.set(id);
        }
    });

    let on_submit = move |_| {
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();
        if !is_edit.get() {
            if kind == UnitKind::Module && system_id.get().is_empty() {
                set_error_msg.set(Some(t_or("supg001.form.error.select_system", "กรุณาเลือกระบบ")));
                return;
            }
            if c.len() < 2 {
                set_error_msg.set(Some(t_or("supg001.form.error.code", "กรุณากรอกรหัส 2-10 ตัวอักษร/ตัวเลข")));
                return;
            }
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("supg001.form.error.name_th", "กรุณากรอกชื่อภาษาไทย")));
            return;
        }
        set_error_msg.set(None);
        ctx.save.run(UnitFormInput {
            system_id: system_id.get(),
            code: c,
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
                        {t_or("supg001.form.protected", "🔒 ใช้เป็นที่ตั้งของหน้าจัดการสิทธิ์: แก้ไขชื่อได้ แต่ปิดใช้งานไม่ได้")}
                    </div>
                })}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    {(kind == UnitKind::Module).then(|| view! {
                        <Field label=t_or("supg001.form.field.system", "ระบบ (System) *")>
                            {move || if is_edit.get() {
                                let label = ctx
                                    .editing_module
                                    .get()
                                    .map(|m| format!("{} {}", m.system_code, m.system_name))
                                    .unwrap_or_default();
                                view! { <ReadOnlyValue value=Signal::derive(move || label.clone()) /> }.into_any()
                            } else {
                                view! {
                                    <Select
                                        value=Signal::derive(move || system_id.get())
                                        on_change=Callback::new(move |v| set_system_id.set(v))
                                        class="bg-background text-sm"
                                    >
                                        {move || ctx.systems.get().into_iter().map(|s| {
                                            let label = s.display_name();
                                            view! { <SelectOption value=s.id.clone()>{label}</SelectOption> }
                                        }).collect_view()}
                                    </Select>
                                }.into_any()
                            }}
                        </Field>
                    })}
                    <Field label=t_or("supg001.form.field.code", "รหัส (Code) *")>
                        {move || if is_edit.get() {
                            view! { <ReadOnlyValue value=Signal::derive(move || code.get()) /> }.into_any()
                        } else {
                            view! {
                                <Input
                                    value=Signal::derive(move || code.get())
                                    on_input=Callback::new(move |v| set_code.set(v))
                                    placeholder=match kind { UnitKind::System => t_or("supg001.form.placeholder.code_system", "เช่น AC"), UnitKind::Module => t_or("supg001.form.placeholder.code_module", "เช่น ACMT") }
                                    class="font-mono text-sm"
                                />
                            }.into_any()
                        }}
                    </Field>
                    <Field label=t_or("supg001.form.field.name_th", "ชื่อ (ภาษาไทย) *")>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |v| set_name_th.set(v))
                            placeholder=t_or("supg001.form.placeholder.name_th", "เช่น บัญชี")
                            class="text-sm"
                        />
                    </Field>
                    <Field label=t_or("supg001.form.field.name_en", "ชื่อ (English)")>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |v| set_name_en.set(v))
                            placeholder="e.g. Accounting"
                            class="text-sm"
                        />
                    </Field>
                </div>

                <div class="pt-2">
                    {move || view! {
                        <Checkbox
                            checked=Signal::derive(move || active.get())
                            on_change=Callback::new(move |v| set_active.set(v))
                            label=t_or("supg001.form.field.active", "เปิดใช้งานในระบบ (Active)")
                            disabled=is_protected.get()
                        />
                    }}
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <Show when=move || is_edit.get() && !is_protected.get() && can_approve.get()>
                        <DeleteConfirmButton
                            label=match kind { UnitKind::System => t_or("supg001.form.delete_system", "ลบระบบ"), UnitKind::Module => t_or("supg001.form.delete_module", "ลบโมดูล") }
                            confirm_text=match kind {
                                UnitKind::System => t_or("supg001.form.confirm_delete_system", "ลบระบบนี้ถาวร?"),
                                UnitKind::Module => t_or("supg001.form.confirm_delete_module", "ลบโมดูลนี้ถาวร?"),
                            }
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
                    <PermissionGate program="SUPG001" action=Action::Write>
                        <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        on_click=Callback::new(on_submit)
                    >
                        {move || if ctx.is_saving.get() { t_or("supg001.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                    </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
