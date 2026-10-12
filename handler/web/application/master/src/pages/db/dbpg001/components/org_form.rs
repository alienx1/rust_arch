//! Organization Form (Create / Edit Organization, DBPG001 ORG)
//!
//! Form for `db_organization` & `db_organization_label`; fields are seeded once the record loads.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, DeleteConfirmButton, Card, CardContent, Checkbox, Input, Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg001::models::CreateOrgInput;
use crate::pages::db::dbpg001::providers::use_dbpg001_org;

#[component]
pub fn OrgForm() -> impl IntoView {
    let ctx = use_dbpg001_org();
    let can_write = can("DBPG001", Action::Write);
    let can_delete = can("DBPG001", Action::Approve);

    let (code, set_code) = signal(String::new());
    let (reg_no, set_reg_no) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Seed the fields when the record to edit has loaded
    Effect::new(move |_| {
        if let Some(org) = ctx.editing.get() {
            set_code.set(org.code);
            set_reg_no.set(org.registration_number);
            set_name_th.set(org.name_th);
            set_name_en.set(org.name_en);
            set_active.set(org.active);
        }
    });

    let on_submit = move |_| {
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();

        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg001.org.validation.code", "กรุณากรอกรหัสองค์กร (Organization Code)")));
            return;
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg001.org.validation.name_th", "กรุณากรอกชื่อองค์กรภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateOrgInput {
            code: c,
            registration_number: reg_no.get().trim().to_string(),
            active: active.get(),
            name_th: th,
            name_en: name_en.get().trim().to_string(),
        });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-4">
                {move || {
                    if ctx.is_loading.get() {
                        view! {
                            <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                                <Spinner class="h-6 w-6 text-primary" />
                                <span class="text-sm">{t_or("dbpg001.org.loading", "กำลังโหลดข้อมูลองค์กร...")}</span>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                {move || {
                    error_msg.get().or_else(|| ctx.error.get()).map(|err| {
                        view! {
                            <div class="p-3 text-xs rounded-md bg-destructive/10 text-destructive border border-destructive/20">
                                {err}
                            </div>
                        }
                    })
                }}

                {move || (!can_write.get()).then(|| view! {
                    <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-400 border border-amber-500/30">
                        {t_or("common.access.read_only", "คุณมีสิทธิ์ดูอย่างเดียว — ไม่สามารถบันทึกหรือแก้ไขข้อมูลได้")}
                    </div>
                })}

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.org.field.code", "รหัสองค์กร (Code) *")}
                        </label>
                        <Input
                            value=Signal::derive(move || code.get())
                            on_input=Callback::new(move |val| set_code.set(val))
                            placeholder=t_or("dbpg001.org.field.code_placeholder", "เช่น ORG001")
                            class="font-mono text-sm"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.org.field.tax_id", "เลขทะเบียนนิติบุคคล / Tax ID")}
                        </label>
                        <Input
                            value=Signal::derive(move || reg_no.get())
                            on_input=Callback::new(move |val| set_reg_no.set(val))
                            placeholder=t_or("dbpg001.org.field.tax_id_placeholder", "เช่น 0105550000001")
                            class="text-sm font-mono"
                        />
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.org.field.name_th", "ชื่อองค์กร (ภาษาไทย) *")}
                        </label>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |val| set_name_th.set(val))
                            placeholder=t_or("dbpg001.org.field.name_th_placeholder", "เช่น บริษัท เอบีซี โฮลดิ้ง จำกัด")
                            class="text-sm"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.org.field.name_en", "ชื่อองค์กร (English)")}
                        </label>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |val| set_name_en.set(val))
                            placeholder="e.g. ABC Holding Co., Ltd."
                            class="text-sm"
                        />
                    </div>
                </div>

                <div class="pt-2">
                    <Checkbox
                        checked=Signal::derive(move || active.get())
                        on_change=Callback::new(move |val| set_active.set(val))
                        label=t_or("dbpg001.form.active", "เปิดใช้งานในระบบ (Active)")
                    />
                </div>

                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <Show when=move || can_delete.get() && ctx.editing.with(|e| e.is_some())>
                        <DeleteConfirmButton
                            label=t_or("dbpg001.org.delete", "ลบองค์กร")
                            confirm_text=t_or("dbpg001.org.delete_confirm", "ลบองค์กรนี้ถาวร?")
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
                    <PermissionGate program="DBPG001" action=Action::Write>
                    <Button
                        variant=ButtonVariant::Default
                        size=ButtonSize::Sm
                        on_click=Callback::new(on_submit)
                    >
                        {move || if ctx.is_saving.get() { t_or("dbpg001.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                    </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
