//! Branch Form (Create / Edit Branch, DBPG002)
//!
//! Form for `db_branch` & `db_branch_label`. The owning company is chosen on create and fixed on
//! edit (departments reference it). Only one headquarter branch is allowed per company; the API
//! rejects a second one and the message is shown here.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_or};
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, DeleteConfirmButton, CardContent, Checkbox, Input, Select, SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg002::models::CreateBranchInput;
use crate::pages::db::dbpg002::providers::use_dbpg002_branch_form;

#[component]
pub fn BranchForm() -> impl IntoView {
    let ctx = use_dbpg002_branch_form();
    let can_write = can("DBPG002", Action::Write);
    let can_approve = can("DBPG002", Action::Approve);

    let (company_id, set_company_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (type_id, set_type_id) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let is_edit = Signal::derive(move || ctx.editing.get().is_some());

    // Seed the fields: record to edit wins, otherwise defaults from the lookups
    Effect::new(move |_| {
        if let Some(b) = ctx.editing.get() {
            set_company_id.set(b.company_id);
            set_code.set(b.code);
            set_type_id.set(b.master_lookup_id);
            set_name_th.set(b.name_th);
            set_name_en.set(b.name_en);
            set_active.set(b.active);
        } else {
            if company_id.get_untracked().is_empty() {
                if let Some(default_company) = ctx.default_company_id.get_untracked() {
                    set_company_id.set(default_company);
                } else if let Some(first) = ctx.companies.get().first() {
                    set_company_id.set(first.id.clone());
                }
            }
            if type_id.get_untracked().is_empty() {
                // Default to an ordinary branch (not the headquarter) when present
                let types = ctx.branch_types.get();
                if let Some(t) = types.iter().find(|t| t.code != "HEAD").or(types.first()) {
                    set_type_id.set(t.id.clone());
                }
            }
        }
    });

    let on_submit = move |_| {
        let company = company_id.get().trim().to_string();
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();
        let ty = type_id.get().trim().to_string();

        if company.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.branch.error.company_required", "กรุณาเลือกบริษัท")));
            return;
        }
        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.branch.error.code_required", "กรุณากรอกรหัสสาขา (Branch Code)")));
            return;
        }
        if ty.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.branch.error.type_required", "กรุณาเลือกประเภทสาขา")));
            return;
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.branch.error.name_th_required", "กรุณากรอกชื่อสาขาภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateBranchInput {
            company_id: company,
            code: c,
            master_lookup_id: ty,
            active: active.get(),
            name_th: th,
            name_en: name_en.get().trim().to_string(),
        });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-4">
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("dbpg002.branch.form.loading", "กำลังโหลดข้อมูลสาขา...")}</span>
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

                <fieldset class="contents" disabled=move || !can_write.get()>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.branch.form.company_label", "บริษัท (Company) *")}</label>
                        {move || if is_edit.get() {
                            let label = ctx
                                .editing
                                .get()
                                .map(|b| format!("{} {}", b.company_code, b.company_name))
                                .unwrap_or_default();
                            view! {
                                <div class="h-9 px-3 flex items-center rounded-md border border-border/60 bg-muted/30 text-sm text-muted-foreground">
                                    {label}
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <Select
                                    value=Signal::derive(move || company_id.get())
                                    on_change=Callback::new(move |v| set_company_id.set(v))
                                    class="bg-background text-sm"
                                >
                                    {move || ctx.companies.get().into_iter().map(|c| {
                                        let label = c.display_name();
                                        view! { <SelectOption value=c.id.clone()>{label}</SelectOption> }
                                    }).collect_view()}
                                </Select>
                            }.into_any()
                        }}
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.branch.form.code_label", "รหัสสาขา (Code) *")}</label>
                        <Input
                            value=Signal::derive(move || code.get())
                            on_input=Callback::new(move |v| set_code.set(v))
                            placeholder=t_or("dbpg002.branch.form.code_placeholder", "เช่น HO, B001")
                            class="font-mono text-sm"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.branch.form.type_label", "ประเภทสาขา (Type) *")}</label>
                        <Select
                            value=Signal::derive(move || type_id.get())
                            on_change=Callback::new(move |v| set_type_id.set(v))
                            class="bg-background text-sm"
                        >
                            {move || ctx.branch_types.get().into_iter().map(|t| view! {
                                <SelectOption value=t.id.clone()>
                                    {format!("{} ({})", localized(&t.name_th, &t.name_en), t.code)}
                                </SelectOption>
                            }).collect_view()}
                        </Select>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.branch.form.name_th_label", "ชื่อสาขา (ภาษาไทย) *")}</label>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |v| set_name_th.set(v))
                            placeholder=t_or("dbpg002.branch.form.name_th_placeholder", "เช่น สำนักงานใหญ่")
                            class="text-sm"
                        />
                    </div>
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.branch.form.name_en_label", "ชื่อสาขา (English)")}</label>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |v| set_name_en.set(v))
                            placeholder="e.g. Head Office"
                            class="text-sm"
                        />
                    </div>
                </div>

                <div class="pt-2">
                    <Checkbox
                        checked=Signal::derive(move || active.get())
                        on_change=Callback::new(move |v| set_active.set(v))
                        label=t_or("dbpg002.form.active", "เปิดใช้งานในระบบ (Active)")
                    />
                </div>
                </fieldset>

                <div class="flex items-center gap-2 pt-2">
                    <Show when=move || can_approve.get()>
                    <Show when=move || ctx.editing.with(|e| e.as_ref().is_some_and(|r| !r.is_headquarter))>
                        <DeleteConfirmButton
                            label=t_or("dbpg002.branch.form.delete", "ลบสาขา")
                            confirm_text=t_or("dbpg002.branch.form.delete_confirm", "ลบสาขานี้ถาวร?")
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
                    <PermissionGate program="DBPG002" action=Action::Write>
                        <Button
                            variant=ButtonVariant::Default
                            size=ButtonSize::Sm
                            on_click=Callback::new(on_submit)
                        >
                            {move || if ctx.is_saving.get() {
                                t_or("dbpg002.form.saving", "กำลังบันทึก...")
                            } else {
                                t_or("common.action.save", "บันทึกข้อมูล")
                            }}
                        </Button>
                    </PermissionGate>
                </div>
            </CardContent>
        </Card>
    }
}
