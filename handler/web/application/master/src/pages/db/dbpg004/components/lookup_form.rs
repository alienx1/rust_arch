//! Lookup Form (Create / Edit reference value, DBPG004)
//!
//! Form for `db_master_lookup` & `db_master_lookup_label`. Category and code are chosen on create
//! and fixed afterwards (other tables and application code reference rows by them). A new
//! category name creates the category. System-protected values cannot be deactivated.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, DeleteConfirmButton, CardContent, Checkbox, Input, Select, SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg004::models::CreateLookupInput;
use crate::pages::db::dbpg004::providers::use_dbpg004_form;

/// Select value meaning "type a new category name"
const NEW_CATEGORY: &str = "__NEW__";

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
pub fn LookupForm() -> impl IntoView {
    let ctx = use_dbpg004_form();
    let can_write = can("DBPG004", Action::Write);

    let (category_choice, set_category_choice) = signal(String::new());
    let (new_category, set_new_category) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = Signal::derive(move || ctx.editing.get().is_some());
    let is_protected = Signal::derive(move || ctx.editing.get().is_some_and(|l| l.is_protected));

    // Seed the fields: record to edit wins, otherwise the category from the list filter
    Effect::new(move |_| {
        if let Some(l) = ctx.editing.get() {
            set_category_choice.set(l.category);
            set_code.set(l.code);
            set_name_th.set(l.name_th);
            set_name_en.set(l.name_en);
            set_active.set(l.active);
        } else if category_choice.get_untracked().is_empty() {
            let categories = ctx.categories.get();
            match ctx.default_category.get_untracked() {
                Some(c) if categories.iter().any(|x| x.category == c) => set_category_choice.set(c),
                _ => {
                    if let Some(first) = categories.iter().find(|c| !c.is_protected) {
                        set_category_choice.set(first.category.clone());
                    }
                }
            }
        }
    });

    let on_submit = move |_| {
        let category = if category_choice.get() == NEW_CATEGORY {
            new_category.get().trim().to_lowercase()
        } else {
            category_choice.get()
        };
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();

        if category.is_empty() {
            set_error_msg.set(Some(t_or("dbpg004.form.err_category_required", "กรุณาเลือกหรือกรอกหมวดหมู่ (Category)")));
            return;
        }
        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg004.form.err_code_required", "กรุณากรอกรหัส (Code)")));
            return;
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg004.form.err_name_th_required", "กรุณากรอกชื่อภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateLookupInput {
            category,
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
                        <span class="text-sm">{t_or("dbpg004.form.loading", "กำลังโหลดข้อมูล...")}</span>
                    </div>
                })}

                {move || error_msg.get().or_else(|| ctx.error.get()).map(|err| view! {
                    <div class="p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20">
                        {err}
                    </div>
                })}

                {move || is_protected.get().then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("dbpg004.form.protected_notice", "🔒 ค่าหลักของระบบ (ใช้ควบคุมสิทธิ์): แก้ไขชื่อได้ แต่ปิดใช้งานไม่ได้")}
                    </div>
                })}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <Field label=t_or("dbpg004.form.category", "หมวดหมู่ (Category) *")>
                        {move || if is_edit.get() {
                            view! { <ReadOnlyValue value=Signal::derive(move || category_choice.get()) /> }.into_any()
                        } else {
                            view! {
                                <Select
                                    value=Signal::derive(move || category_choice.get())
                                    on_change=Callback::new(move |v| set_category_choice.set(v))
                                    class="bg-background text-sm"
                                >
                                    {move || ctx.categories.get().into_iter().filter(|c| !c.is_protected).map(|c| {
                                        let label = t_fmt("dbpg004.form.category_option", "{category} ({count})", &[("category", c.category.as_str()), ("count", &c.total.to_string())]);
                                        view! { <SelectOption value=c.category.clone()>{label}</SelectOption> }
                                    }).collect_view()}
                                    <SelectOption value=NEW_CATEGORY.to_string()>{t_or("dbpg004.form.new_category_option", "➕ สร้างหมวดหมู่ใหม่...")}</SelectOption>
                                </Select>
                            }.into_any()
                        }}
                    </Field>

                    {move || (!is_edit.get() && category_choice.get() == NEW_CATEGORY).then(|| view! {
                        <Field label=t_or("dbpg004.form.new_category", "ชื่อหมวดหมู่ใหม่ * (a-z, 0-9, _)")>
                            <Input
                                value=Signal::derive(move || new_category.get())
                                on_input=Callback::new(move |v| set_new_category.set(v))
                                placeholder=t_or("dbpg004.form.new_category_placeholder", "เช่น payment_method")
                                class="font-mono text-sm"
                            />
                        </Field>
                    })}

                    <Field label=t_or("dbpg004.form.code", "รหัส (Code) *")>
                        {move || if is_edit.get() {
                            view! { <ReadOnlyValue value=Signal::derive(move || code.get()) /> }.into_any()
                        } else {
                            view! {
                                <Input
                                    value=Signal::derive(move || code.get())
                                    on_input=Callback::new(move |v| set_code.set(v))
                                    placeholder=t_or("dbpg004.form.code_placeholder", "เช่น CASH")
                                    class="font-mono text-sm"
                                />
                            }.into_any()
                        }}
                    </Field>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <Field label=t_or("dbpg004.form.name_th", "ชื่อ (ภาษาไทย) *")>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |v| set_name_th.set(v))
                            placeholder=t_or("dbpg004.form.name_th_placeholder", "เช่น เงินสด")
                            class="text-sm"
                        />
                    </Field>
                    <Field label=t_or("dbpg004.form.name_en", "ชื่อ (English)")>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |v| set_name_en.set(v))
                            placeholder="e.g. Cash"
                            class="text-sm"
                        />
                    </Field>
                </div>

                <div class="pt-2">
                    // Re-rendered when the loaded record turns out to be system-protected
                    {move || view! {
                        <Checkbox
                            checked=Signal::derive(move || active.get())
                            on_change=Callback::new(move |v| set_active.set(v))
                            label=t_or("dbpg004.form.active", "เปิดใช้งานในระบบ (Active)")
                            disabled=is_protected.get()
                        />
                    }}
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <PermissionGate program="DBPG004" action=Action::Approve>
                    <Show when=move || ctx.editing.with(|e| e.as_ref().is_some_and(|l| !l.is_protected))>
                        <DeleteConfirmButton
                            label=t_or("dbpg004.form.delete", "ลบค่าอ้างอิง")
                            confirm_text=t_or("dbpg004.form.delete_confirm", "ลบค่าอ้างอิงนี้ถาวร?")
                            on_confirm=ctx.delete
                            busy=Signal::derive(move || ctx.is_deleting.get() || ctx.is_saving.get())
                        />
                    </Show>
                    </PermissionGate>
                    <div class="flex-1" />
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| ctx.cancel.run(()))
                    >
                        {t_or("common.action.cancel", "ยกเลิก")}
                    </Button>
                    <PermissionGate program="DBPG004" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        on_click=Callback::new(on_submit)
                    >
                        {move || if ctx.is_saving.get() { t_or("dbpg004.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                    </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
