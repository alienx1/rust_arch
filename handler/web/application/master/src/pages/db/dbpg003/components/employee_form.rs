//! Employee Form (Create / Edit Employee, DBPG003)
//!
//! Form for `db_employee`, `db_person_name` & `db_person_name_label`. Company → branch →
//! department are cascading selectors; only the department is submitted (the server derives
//! organization, company and branch from it). English names are optional on create (the Thai name
//! is copied) and left unchanged on edit when blank.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{SaveConfirmButton, 
    ContactsEditor, ImageField,
    Button, ButtonSize, ButtonVariant, Card, DeleteConfirmButton, CardContent, Checkbox, Input, Select, SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg003::models::CreateEmployeeInput;
use crate::pages::db::dbpg003::providers::use_dbpg003_form;

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
fn TextField(
    #[prop(into)] label: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    view! {
        <Field label=label>
            <Input
                value=Signal::derive(move || value.get())
                on_input=Callback::new(move |v| set_value.set(v))
                placeholder=placeholder
                class=format!("text-sm {class}")
            />
        </Field>
    }
}

#[component]
pub fn EmployeeForm() -> impl IntoView {
    let ctx = use_dbpg003_form();
    let can_write = can("DBPG003", Action::Write);
    let can_approve = can("DBPG003", Action::Approve);

    let (company_id, set_company_id) = signal(String::new());
    let (branch_id, set_branch_id) = signal(String::new());
    let (department_id, set_department_id) = signal(String::new());
    let (prefix_id, set_prefix_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (first_th, set_first_th) = signal(String::new());
    let (middle_th, set_middle_th) = signal(String::new());
    let (last_th, set_last_th) = signal(String::new());
    let (nick_th, set_nick_th) = signal(String::new());
    let (first_en, set_first_en) = signal(String::new());
    let (middle_en, set_middle_en) = signal(String::new());
    let (last_en, set_last_en) = signal(String::new());
    let (nick_en, set_nick_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let branches_of = move |company: &str| -> Vec<_> {
        ctx.branches.get().into_iter().filter(|b| b.company_id == company).collect()
    };
    let departments_of = move |branch: &str| -> Vec<_> {
        ctx.departments.get().into_iter().filter(|d| d.branch_id == branch).collect()
    };

    // Seed the fields: record to edit wins, otherwise defaults from the lookups
    Effect::new(move |_| {
        if let Some(e) = ctx.editing.get() {
            set_company_id.set(e.company_id);
            set_branch_id.set(e.branch_id);
            set_department_id.set(e.department_id);
            set_prefix_id.set(e.prefix_id);
            set_code.set(e.code);
            set_first_th.set(e.first_name_th);
            set_middle_th.set(e.middle_name_th);
            set_last_th.set(e.last_name_th);
            set_nick_th.set(e.nickname_th);
            set_first_en.set(e.first_name_en);
            set_middle_en.set(e.middle_name_en);
            set_last_en.set(e.last_name_en);
            set_nick_en.set(e.nickname_en);
            set_active.set(e.active);
        } else {
            if company_id.get_untracked().is_empty() {
                if let Some(default_company) = ctx.default_company_id.get_untracked() {
                    set_company_id.set(default_company);
                } else if let Some(first) = ctx.companies.get().first() {
                    set_company_id.set(first.id.clone());
                }
            }
            let company = company_id.get_untracked();
            if branch_id.get_untracked().is_empty() {
                if let Some(first) = branches_of(&company).first() {
                    set_branch_id.set(first.id.clone());
                }
            }
            let branch = branch_id.get_untracked();
            if department_id.get_untracked().is_empty() {
                if let Some(first) = departments_of(&branch).first() {
                    set_department_id.set(first.id.clone());
                }
            }
            if prefix_id.get_untracked().is_empty() {
                if let Some(first) = ctx.prefixes.get().first() {
                    set_prefix_id.set(first.id.clone());
                }
            }
        }
    });

    let on_company = Callback::new(move |v: String| {
        let first_branch = branches_of(&v).first().map(|b| b.id.clone()).unwrap_or_default();
        let first_dept = departments_of(&first_branch).first().map(|d| d.id.clone()).unwrap_or_default();
        set_company_id.set(v);
        set_branch_id.set(first_branch);
        set_department_id.set(first_dept);
    });
    let on_branch = Callback::new(move |v: String| {
        let first_dept = departments_of(&v).first().map(|d| d.id.clone()).unwrap_or_default();
        set_branch_id.set(v);
        set_department_id.set(first_dept);
    });

    let on_submit = move |_| {
        let c = code.get().trim().to_uppercase();
        let dept = department_id.get().trim().to_string();
        let prefix = prefix_id.get().trim().to_string();

        if dept.is_empty() {
            set_error_msg.set(Some(t_or("dbpg003.form.error.department_required", "กรุณาเลือกแผนก (ต้องมีแผนกใน DBPG002 ก่อนจึงจะเพิ่มพนักงานได้)")));
            return;
        }
        if prefix.is_empty() {
            set_error_msg.set(Some(t_or("dbpg003.form.error.prefix_required", "กรุณาเลือกคำนำหน้าชื่อ")));
            return;
        }
        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg003.form.error.code_required", "กรุณากรอกรหัสพนักงาน (Employee Code)")));
            return;
        }
        if first_th.get().trim().is_empty() || last_th.get().trim().is_empty() {
            set_error_msg.set(Some(t_or("dbpg003.form.error.name_th_required", "กรุณากรอกชื่อและนามสกุลภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateEmployeeInput {
            department_id: dept,
            prefix_id: prefix,
            code: c,
            active: active.get(),
            first_name_th: first_th.get().trim().to_string(),
            middle_name_th: middle_th.get().trim().to_string(),
            last_name_th: last_th.get().trim().to_string(),
            nickname_th: nick_th.get().trim().to_string(),
            first_name_en: first_en.get().trim().to_string(),
            middle_name_en: middle_en.get().trim().to_string(),
            last_name_en: last_en.get().trim().to_string(),
            nickname_en: nick_en.get().trim().to_string(),
        });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-5">
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("dbpg003.form.loading", "กำลังโหลดข้อมูลพนักงาน...")}</span>
                    </div>
                })}

                {move || error_msg.get().or_else(|| ctx.error.get()).map(|err| view! {
                    <div class="p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20">
                        {err}
                    </div>
                })}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-400 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <ImageField
                    image_url=Signal::derive(move || ctx.editing.with(|e| e.as_ref().and_then(|x| x.image_url.clone())))
                    saved=Signal::derive(move || ctx.editing.with(|e| e.is_some()))
                    busy=ctx.is_image_busy
                    can_write=can_write
                    fallback=Signal::derive(move || ctx.editing.with(|e| e.as_ref().map(|x| x.full_name.clone()).unwrap_or_default()))
                    on_upload=ctx.upload_image
                    on_remove=ctx.remove_image
                />

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("dbpg003.form.section_org", "สังกัด (Organization)")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <Field label=t_or("dbpg003.form.company_label", "บริษัท (Company) *")>
                            <Select
                                value=Signal::derive(move || company_id.get())
                                on_change=on_company
                                class="bg-background text-sm"
                            >
                                {move || ctx.companies.get().into_iter().map(|c| {
                                    let label = c.display_name();
                                    view! { <SelectOption value=c.id.clone()>{label}</SelectOption> }
                                }).collect_view()}
                            </Select>
                        </Field>
                        <Field label=t_or("dbpg003.form.branch_label", "สาขา (Branch) *")>
                            <Select
                                value=Signal::derive(move || branch_id.get())
                                on_change=on_branch
                                class="bg-background text-sm"
                            >
                                {move || branches_of(&company_id.get()).into_iter().map(|b| {
                                    let label = b.display_name();
                                    view! { <SelectOption value=b.id.clone()>{label}</SelectOption> }
                                }).collect_view()}
                            </Select>
                        </Field>
                        <Field label=t_or("dbpg003.form.department_label", "แผนก (Department) *")>
                            <Select
                                value=Signal::derive(move || department_id.get())
                                on_change=Callback::new(move |v| set_department_id.set(v))
                                class="bg-background text-sm"
                            >
                                {move || departments_of(&branch_id.get()).into_iter().map(|d| {
                                    let label = d.display_name();
                                    view! { <SelectOption value=d.id.clone()>{label}</SelectOption> }
                                }).collect_view()}
                            </Select>
                        </Field>
                    </div>
                </div>

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("dbpg003.form.section_employee", "ข้อมูลพนักงาน (Employee)")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <TextField
                            label=t_or("dbpg003.form.code_label", "รหัสพนักงาน (Code) *")
                            value=code
                            set_value=set_code
                            placeholder=t_or("dbpg003.form.code_placeholder", "เช่น E0001")
                            class="font-mono"
                        />
                        <Field label=t_or("dbpg003.form.prefix_label", "คำนำหน้าชื่อ (Prefix) *")>
                            <Select
                                value=Signal::derive(move || prefix_id.get())
                                on_change=Callback::new(move |v| set_prefix_id.set(v))
                                class="bg-background text-sm"
                            >
                                {move || ctx.prefixes.get().into_iter().map(|p| view! {
                                    <SelectOption value=p.id.clone()>{localized(&p.name_th, &p.name_en)}</SelectOption>
                                }).collect_view()}
                            </Select>
                        </Field>
                    </div>
                </div>

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{t_or("dbpg003.form.section_name_th", "ชื่อ-นามสกุล (ภาษาไทย)")}</h3>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                        <TextField label=t_or("dbpg003.form.first_name_label", "ชื่อ *") value=first_th set_value=set_first_th placeholder=t_or("dbpg003.form.first_name_placeholder", "สมชาย") />
                        <TextField label=t_or("dbpg003.form.middle_name_label", "ชื่อกลาง") value=middle_th set_value=set_middle_th />
                        <TextField label=t_or("dbpg003.form.last_name_label", "นามสกุล *") value=last_th set_value=set_last_th placeholder=t_or("dbpg003.form.last_name_placeholder", "ใจดี") />
                        <TextField label=t_or("dbpg003.form.nickname_label", "ชื่อเล่น") value=nick_th set_value=set_nick_th />
                    </div>
                </div>

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">"Name (English)"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
                        <TextField label="First name" value=first_en set_value=set_first_en placeholder="Somchai" />
                        <TextField label="Middle name" value=middle_en set_value=set_middle_en />
                        <TextField label="Last name" value=last_en set_value=set_last_en placeholder="Jaidee" />
                        <TextField label="Nickname" value=nick_en set_value=set_nick_en />
                    </div>
                </div>

                <div class="pt-2">
                    <Checkbox
                        checked=Signal::derive(move || active.get())
                        on_change=Callback::new(move |v| set_active.set(v))
                        label=t_or("dbpg003.form.active", "กำลังปฏิบัติงาน (Active)")
                    />
                </div>
                <ContactsEditor
                    rows=ctx.contacts
                    kinds=vec!["PHONE", "EMAIL", "LINE", "FAX", "ID_CARD"]
                    disabled=Signal::derive(move || !can_write.get())
                />

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <Show when=move || can_approve.get()>
                    <Show when=move || ctx.editing.with(|e| e.is_some())>
                        <DeleteConfirmButton
                            label=t_or("dbpg003.form.delete", "ลบพนักงาน")
                            confirm_text=t_or("dbpg003.form.delete_confirm", "ลบพนักงานนี้ถาวร?")
                            on_confirm=ctx.delete
                            busy=Signal::derive(move || ctx.is_deleting.get() || ctx.is_saving.get())
                        />
                    </Show>
                    </Show>
                    <div class="flex-1" />
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| ctx.cancel.run(()))
                    >
                        {t_or("common.action.cancel", "ยกเลิก")}
                    </Button>
                    <PermissionGate program="DBPG003" action=Action::Write>
                        <SaveConfirmButton
                            label=Signal::derive(move || if ctx.is_saving.get() {
                                t_or("dbpg003.form.saving", "กำลังบันทึก...")
                            } else {
                                t_or("common.action.save", "บันทึกข้อมูล")
                            })
                            on_confirm=Callback::new(on_submit)
                            busy=Signal::derive(move || ctx.is_saving.get())
                        />
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
