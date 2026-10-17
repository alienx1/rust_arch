//! Permission group form (SUPG003): code and status only.
//!
//! A group has no scope: the scope is chosen per user in SUPG004 (a user's assignments). The code
//! is fixed after creation (users are bound to it). The permissions themselves are set in the matrix.

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{SaveConfirmButton, 
    Button, ButtonSize, ButtonVariant, Card, CardContent, Checkbox, DeleteConfirmButton, Input, Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::su::supg003::models::GroupFormInput;
use crate::pages::su::supg003::providers::use_supg003_form;

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
pub fn GroupForm() -> impl IntoView {
    let ctx = use_supg003_form();
    let can_write = can("SUPG003", Action::Write);

    let (code, set_code) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let is_edit = Signal::derive(move || ctx.editing.get().is_some());
    let is_protected = Signal::derive(move || ctx.editing.get().is_some_and(|g| g.is_protected));

    Effect::new(move |_| {
        if let Some(g) = ctx.editing.get() {
            set_code.set(g.code);
            set_active.set(g.active);
        }
    });

    let on_submit = move |_| {
        let c = code.get().trim().to_uppercase();
        if !is_edit.get() && c.len() < 2 {
            set_error_msg.set(Some(t_or("supg003.form.err_code", "กรุณากรอกรหัสกลุ่มสิทธิ์ (2-50 ตัวอักษร/ตัวเลข/_)")));
            return;
        }
        set_error_msg.set(None);
        ctx.save.run(GroupFormInput {
            code: c,
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

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                {move || is_protected.get().then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                        {t_or("supg003.form.protected_notice", "🔒 กลุ่มผู้ดูแลระบบ: เข้าถึงได้ทุกโปรแกรม ปิดใช้งานไม่ได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <Field label=t_or("supg003.form.code", "รหัสกลุ่มสิทธิ์ (Code) *")>
                        {move || if is_edit.get() {
                            view! { <ReadOnlyValue value=Signal::derive(move || code.get()) /> }.into_any()
                        } else {
                            view! {
                                <Input
                                    value=Signal::derive(move || code.get())
                                    on_input=Callback::new(move |v| set_code.set(v))
                                    placeholder=t_or("supg003.form.code_placeholder", "เช่น HR_MANAGER")
                                    class="font-mono text-sm"
                                />
                            }.into_any()
                        }}
                    </Field>
                </div>

                <div class="pt-2">
                    {move || view! {
                        <Checkbox
                            checked=Signal::derive(move || active.get())
                            on_change=Callback::new(move |v| set_active.set(v))
                            label=t_or("supg003.form.active", "เปิดใช้งานกลุ่มสิทธิ์ (Active)")
                            disabled=is_protected.get()
                        />
                    }}
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    {move || (is_edit.get() && !is_protected.get()).then(|| view! {
                        <PermissionGate program="SUPG003" action=Action::Approve>
                        <DeleteConfirmButton
                            label=t_or("supg003.form.delete", "ลบกลุ่มสิทธิ์")
                            confirm_text=t_or("supg003.form.delete_confirm", "ลบกลุ่มสิทธิ์นี้ถาวร (รวมเมทริกซ์สิทธิ์)?")
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
                    <PermissionGate program="SUPG003" action=Action::Write>
                    <SaveConfirmButton
                        label=Signal::derive(move || if ctx.is_saving.get() { t_or("supg003.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") })
                        on_confirm=Callback::new(on_submit)
                        busy=Signal::derive(move || ctx.is_saving.get())
                    />
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
