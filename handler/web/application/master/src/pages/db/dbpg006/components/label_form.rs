//! Label Form (Create / Edit i18n label, DBPG006)
//!
//! Form for `db_label`. Category and key are chosen on create and fixed afterwards (application
//! code looks texts up by them); a new category name creates the category. One text input per
//! active language: a blank text means "no translation" for that language.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{SaveConfirmButton, 
    Button, ButtonSize, ButtonVariant, Card, CardContent, DeleteConfirmButton, Input, Select,
    SelectOption, Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg006::models::{LabelValueInput, SaveLabelInput};
use crate::pages::db::dbpg006::providers::use_dbpg006_form;

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
        <div class="min-h-9 px-3 py-2 flex items-center rounded-md border border-border/60 bg-muted/30 text-sm font-mono text-muted-foreground break-all">
            {move || value.get()}
        </div>
    }
}

#[component]
pub fn LabelForm() -> impl IntoView {
    let ctx = use_dbpg006_form();
    let can_write = can("DBPG006", Action::Write);

    let (category_choice, set_category_choice) = signal(String::new());
    let (new_category, set_new_category) = signal(String::new());
    let (key, set_key) = signal(String::new());
    // One text signal per active language, (language id, code, Thai name, text)
    let texts = RwSignal::new(Vec::<(String, String, String, RwSignal<String>)>::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = ctx.target.get_untracked().is_some();

    // Fields of the entry to edit, or the category chosen in the list filter
    Effect::new(move |_| {
        let editing = ctx.editing.get();
        let rows = ctx
            .languages
            .get()
            .into_iter()
            .map(|l| {
                let text = editing
                    .as_ref()
                    .and_then(|e| e.values.iter().find(|v| v.language_id == l.id))
                    .map(|v| v.value.clone())
                    .unwrap_or_default();
                (l.id, l.code, l.name_th, RwSignal::new(text))
            })
            .collect();
        texts.set(rows);
    });
    Effect::new(move |_| {
        if let Some(e) = ctx.editing.get() {
            set_category_choice.set(e.category);
            set_key.set(e.key);
        } else if category_choice.get_untracked().is_empty() {
            let categories = ctx.categories.get();
            match ctx.default_category.get_untracked() {
                Some(c) if categories.iter().any(|x| x.category == c) => set_category_choice.set(c),
                Some(c) if !c.is_empty() => {
                    set_new_category.set(c);
                    set_category_choice.set(NEW_CATEGORY.to_string());
                }
                _ => {
                    if let Some(first) = categories.first() {
                        set_category_choice.set(first.category.clone());
                    }
                }
            }
        }
    });

    let on_submit = move |_| {
        let category = if category_choice.get() == NEW_CATEGORY {
            new_category.get().trim().to_string()
        } else {
            category_choice.get()
        };
        let k = key.get().trim().to_string();
        let values: Vec<LabelValueInput> = texts
            .get()
            .into_iter()
            .map(|(id, _, _, text)| LabelValueInput { language_id: id, value: text.get().trim().to_string() })
            .collect();

        if category.is_empty() {
            set_error_msg.set(Some(t_or("dbpg006.form.err_category_required", "กรุณาเลือกหรือกรอกหมวด (Category)")));
            return;
        }
        if !category.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')) {
            set_error_msg.set(Some(t_or("dbpg006.form.err_category_chars", "หมวดใช้ได้เฉพาะ a-z, A-Z, 0-9, _ . -")));
            return;
        }
        if k.is_empty() {
            set_error_msg.set(Some(t_or("dbpg006.form.err_key_required", "กรุณากรอกคีย์ (Key)")));
            return;
        }
        if values.iter().all(|v| v.value.is_empty()) {
            set_error_msg.set(Some(t_or("dbpg006.form.err_text_required", "กรุณากรอกข้อความอย่างน้อย 1 ภาษา")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(SaveLabelInput { category, key: k, values, row_version: 0 });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-5">
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("dbpg006.form.loading", "กำลังโหลดข้อมูล...")}</span>
                    </div>
                })}

                {move || error_msg.get().or_else(|| ctx.error.get()).map(|err| view! {
                    <div class="p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20">
                        {err}
                    </div>
                })}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <Field label=t_or("dbpg006.form.category", "หมวด (Category) *")>
                        {if is_edit {
                            view! { <ReadOnlyValue value=Signal::derive(move || category_choice.get()) /> }.into_any()
                        } else {
                            view! {
                                <Select
                                    value=Signal::derive(move || category_choice.get())
                                    on_change=Callback::new(move |v| set_category_choice.set(v))
                                    class="bg-background text-sm"
                                >
                                    {move || ctx.categories.get().into_iter().map(|c| {
                                        let label = t_fmt("dbpg006.form.category_option", "{category} ({count})", &[("category", c.category.as_str()), ("count", &c.entries.to_string())]);
                                        view! { <SelectOption value=c.category.clone()>{label}</SelectOption> }
                                    }).collect_view()}
                                    <SelectOption value=NEW_CATEGORY.to_string()>{t_or("dbpg006.form.new_category_option", "➕ สร้างหมวดใหม่...")}</SelectOption>
                                </Select>
                            }.into_any()
                        }}
                    </Field>

                    {move || (!is_edit && category_choice.get() == NEW_CATEGORY).then(|| view! {
                        <Field label=t_or("dbpg006.form.new_category", "ชื่อหมวดใหม่ * (a-z, 0-9, _ . -)")>
                            <Input
                                value=Signal::derive(move || new_category.get())
                                on_input=Callback::new(move |v| set_new_category.set(v))
                                placeholder=t_or("dbpg006.form.new_category_placeholder", "เช่น supg006")
                                class="font-mono text-sm"
                            />
                        </Field>
                    })}

                    <Field label=t_or("dbpg006.form.key", "คีย์ (Key) *")>
                        {if is_edit {
                            view! { <ReadOnlyValue value=Signal::derive(move || key.get()) /> }.into_any()
                        } else {
                            view! {
                                <Input
                                    value=Signal::derive(move || key.get())
                                    on_input=Callback::new(move |v| set_key.set(v))
                                    placeholder=t_or("dbpg006.form.key_placeholder", "เช่น column.thai_name")
                                    class="font-mono text-sm"
                                />
                            }.into_any()
                        }}
                    </Field>
                </div>

                <div class="space-y-3">
                    <p class="text-xs text-muted-foreground">
                        {t_or("dbpg006.form.texts_hint", "ข้อความแปลแต่ละภาษา (เว้นว่าง = ไม่มีข้อความสำหรับภาษานั้น • ต้องมีอย่างน้อย 1 ภาษา)")}
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        {move || texts.get().into_iter().map(|(_, code, name_th, text)| view! {
                            <div class="space-y-1.5">
                                <label class="text-xs font-semibold text-muted-foreground">
                                    {format!("{name_th} ({code})")}
                                </label>
                                <Input
                                    value=Signal::derive(move || text.get())
                                    on_input=Callback::new(move |v| text.set(v))
                                    class="text-sm"
                                />
                            </div>
                        }).collect_view()}
                    </div>
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <PermissionGate program="DBPG006" action=Action::Approve>
                    <Show when=move || is_edit>
                        <DeleteConfirmButton
                            label=t_or("dbpg006.form.delete", "ลบข้อความ")
                            confirm_text=t_or("dbpg006.form.delete_confirm", "ลบข้อความนี้ทุกภาษาถาวร?")
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
                    <PermissionGate program="DBPG006" action=Action::Write>
                    <SaveConfirmButton
                        label=Signal::derive(move || if ctx.is_saving.get() { t_or("dbpg006.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") })
                        on_confirm=Callback::new(on_submit)
                        busy=Signal::derive(move || ctx.is_saving.get())
                    />
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
