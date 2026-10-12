//! Account form (SUPG004): employee (searchable), email, username, assignments and status.
//!
//! Employee, email and username are chosen on create and fixed afterwards (the identity provider
//! and history reference them); editing replaces its assignments (group @ scope unit) or suspends it.
//! After a successful create the form is replaced by the one-time recovery link.

use web_package::core::i18n::{t_or, t_fmt};
use leptos::prelude::*;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Checkbox, DeleteConfirmButton, Input, Select,
    SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::su::supg004::components::{AssignmentEditor, RecoveryLinkPanel};
use crate::pages::su::supg004::models::{UserAssignmentDto, UserFormInput, username_from_email};
use crate::pages::su::supg004::providers::use_supg004_form;

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
        <div class="h-9 px-3 flex items-center rounded-md border border-border/60 bg-muted/30 text-sm text-muted-foreground">
            {move || value.get()}
        </div>
    }
}

#[component]
pub fn UserForm() -> impl IntoView {
    let ctx = use_supg004_form();
    let can_write = can("SUPG004", Action::Write);

    let (employee_id, set_employee_id) = signal(String::new());
    let (employee_search, set_employee_search) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (username_touched, set_username_touched) = signal(false);
    let (assignments, set_assignments) = signal(Vec::<UserAssignmentDto>::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = Signal::derive(move || ctx.editing.get().is_some());

    Effect::new(move |_| {
        if let Some(u) = ctx.editing.get() {
            set_assignments.set(u.assignments);
            set_active.set(u.active);
        }
    });

    // Create: preselect the first employee / group once they arrive
    Effect::new(move |_| {
        if is_edit.get() {
            return;
        }
        let list = ctx.employees.get();
        if !list.iter().any(|e| e.id == employee_id.get_untracked()) {
            set_employee_id.set(list.first().map(|e| e.id.clone()).unwrap_or_default());
        }
    });
    let on_submit = move |_| {
        if !is_edit.get() {
            if employee_id.get().is_empty() {
                set_error_msg.set(Some(t_or("supg004.form.err_employee", "กรุณาเลือกพนักงาน (ไม่มีพนักงานที่ยังไม่มีบัญชี)")));
                return;
            }
            let e = email.get();
            if !e.contains('@') || e.trim().len() < 5 {
                set_error_msg.set(Some(t_or("supg004.form.err_email", "กรุณากรอกอีเมลให้ถูกต้อง")));
                return;
            }
            if username.get().trim().len() < 3 {
                set_error_msg.set(Some(t_or("supg004.form.err_username", "ชื่อผู้ใช้ต้องมี 3-64 ตัวอักษร (a-z 0-9 . _ -)")));
                return;
            }
        }
        if assignments.get().is_empty() {
            set_error_msg.set(Some(t_or("supg004.form.err_assignment_required", "กรุณาเพิ่มกลุ่มสิทธิ์และขอบเขตอย่างน้อย 1 รายการ")));
            return;
        }
        set_error_msg.set(None);
        ctx.save.run(UserFormInput {
            employee_id: employee_id.get(),
            username: username.get().trim().to_lowercase(),
            email: email.get().trim().to_string(),
            assignments: assignments.get().iter().map(UserAssignmentDto::command).collect(),
            active: active.get(),
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

                {move || (!can_write.get() && ctx.created.get().is_none()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                {move || ctx.created.get().map(|result| {
                    let username = result.user.username.clone();
                    view! {
                        <div class="space-y-3">
                            <div class="p-3 text-xs rounded-md bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                                {t_fmt("supg004.form.created", "สร้างบัญชี {username} เรียบร้อยแล้ว", &[("username", &username.to_string())])}
                            </div>
                            {match result.recovery_link {
                                Some(link) => view! { <RecoveryLinkPanel username=username link=link /> }.into_any(),
                                None => view! {
                                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                                        {t_fmt("supg004.form.recovery_failed", "ยังออกลิงก์ตั้งรหัสผ่านไม่สำเร็จ ({error}) — ออกใหม่ได้จากหน้ารายการ", &[("error", &result.recovery_error.unwrap_or_default().to_string())])}
                                    </div>
                                }.into_any(),
                            }}
                            <div class="flex justify-end">
                                <Button variant=ButtonVariant::Default size=ButtonSize::Sm on_click=Callback::new(move |_| ctx.cancel.run(()))>
                                    {t_or("supg004.form.back_to_list", "กลับไปหน้ารายการ")}
                                </Button>
                            </div>
                        </div>
                    }
                })}

                {move || ctx.created.get().is_none().then(|| view! {
                    <div class="space-y-5">
                        <fieldset class="contents" disabled=move || !can_write.get()>
                        {move || if is_edit.get() {
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                                    <Field label=t_or("supg004.form.username", "ชื่อผู้ใช้ (Username)")>
                                        <ReadOnlyValue value=Signal::derive(move || ctx.editing.get().map(|u| u.username).unwrap_or_default()) />
                                    </Field>
                                    <div class="md:col-span-2">
                                        <Field label=t_or("supg004.form.employee", "พนักงาน")>
                                            <ReadOnlyValue value=Signal::derive(move || {
                                                ctx.editing.get()
                                                    .map(|u| format!("{} • {} ({})", u.employee_code, u.full_name, u.department_name))
                                                    .unwrap_or_default()
                                            }) />
                                        </Field>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <Field label=t_or("supg004.form.employee_search", "ค้นหาพนักงาน")>
                                        <Input
                                            value=Signal::derive(move || employee_search.get())
                                            on_input=Callback::new(move |v: String| {
                                                set_employee_search.set(v.clone());
                                                ctx.search_employees.run(v);
                                            })
                                            placeholder=t_or("supg004.form.employee_search_placeholder", "รหัสหรือชื่อพนักงาน (เฉพาะที่ยังไม่มีบัญชี)")
                                            class="text-sm"
                                        />
                                    </Field>
                                    <Field label=t_or("supg004.form.employee_required", "พนักงาน *")>
                                        <Select
                                            value=Signal::derive(move || employee_id.get())
                                            on_change=Callback::new(move |v| set_employee_id.set(v))
                                            class="bg-background text-sm"
                                        >
                                            {move || ctx.employees.get().into_iter().map(|e| {
                                                let label = e.display_name();
                                                view! { <SelectOption value=e.id.clone()>{label}</SelectOption> }
                                            }).collect_view()}
                                        </Select>
                                    </Field>
                                    <Field label=t_or("supg004.form.email", "อีเมล (ใช้เข้าสู่ระบบ/กู้คืนรหัสผ่าน) *")>
                                        <Input
                                            value=Signal::derive(move || email.get())
                                            on_input=Callback::new(move |v: String| {
                                                if !username_touched.get_untracked() {
                                                    set_username.set(username_from_email(&v));
                                                }
                                                set_email.set(v);
                                            })
                                            placeholder="name@company.com"
                                            class="text-sm"
                                        />
                                    </Field>
                                    <Field label=t_or("supg004.form.username_required", "ชื่อผู้ใช้ (Username) *")>
                                        <Input
                                            value=Signal::derive(move || username.get())
                                            on_input=Callback::new(move |v: String| {
                                                set_username_touched.set(true);
                                                set_username.set(v);
                                            })
                                            placeholder=t_or("supg004.form.username_placeholder", "เช่น somchai.j")
                                            class="font-mono text-sm"
                                        />
                                    </Field>
                                </div>
                            }.into_any()
                        }}

                        <AssignmentEditor
                            assignments=assignments
                            set_assignments=set_assignments
                            on_error=Callback::new(move |m| set_error_msg.set(m))
                        />

                        <div class="pt-2">
                            <Checkbox
                                checked=Signal::derive(move || active.get())
                                on_change=Callback::new(move |v| set_active.set(v))
                                label=t_or("supg004.form.active", "เปิดใช้งานบัญชี (ยกเลิก = ระงับการเข้าสู่ระบบและเซสชันทั้งหมด)")
                            />
                        </div>

                        </fieldset>

                        <div class="flex items-center gap-2 pt-2">
                            {move || is_edit.get().then(|| view! {
                                <PermissionGate program="SUPG004" action=Action::Approve>
                                <DeleteConfirmButton
                                    label=t_or("supg004.form.delete", "ลบบัญชี")
                                    confirm_text=t_or("supg004.form.delete_confirm", "ลบบัญชีนี้ถาวร (ข้อมูลการเข้าสู่ระบบ/identity จะถูกลบด้วย)?")
                                    on_confirm=ctx.delete
                                    busy=Signal::derive(move || ctx.is_deleting.get() || ctx.is_saving.get())
                                />
                                </PermissionGate>
                            })}
                            <div class="flex-1" />
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                on_click=Callback::new(move |_| ctx.cancel.run(()))
                            >
                                {t_or("common.action.cancel", "ยกเลิก")}
                            </Button>
                            <PermissionGate program="SUPG004" action=Action::Write>
                                <Button
                                    variant=ButtonVariant::Default
                                    size=ButtonSize::Sm
                                    on_click=Callback::new(on_submit)
                                >
                                    {move || if ctx.is_saving.get() { t_or("supg004.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                                </Button>
                            </PermissionGate>
                        </div>
                    </div>
                })}
            </CardContent>
        </Card>
    }
}
