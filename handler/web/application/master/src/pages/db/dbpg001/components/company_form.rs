//! Company Form (Create / Edit Company, DBPG001 COMPANY)
//!
//! Form for `db_company` & `db_company_label`; fields are seeded once the lookups and the
//! record to edit have loaded.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, DeleteConfirmButton, Card, CardContent, Checkbox, Input, Select, SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg001::models::CreateCompanyInput;
use crate::pages::db::dbpg001::providers::use_dbpg001_company;

/// Fallback company type when the type lookup is empty (PRIVATE).
const DEFAULT_COMPANY_TYPE_ID: &str = "019ca1eb-54c0-7000-8007-000000000002";

#[component]
pub fn CompanyForm() -> impl IntoView {
    let ctx = use_dbpg001_company();
    let can_write = can("DBPG001", Action::Write);
    let can_delete = can("DBPG001", Action::Approve);
    let orgs = ctx.organizations;
    let company_types = ctx.company_types;

    let (org_id, set_org_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (lookup_id, set_lookup_id) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Seed the fields: record to edit wins, otherwise defaults from the lookups
    Effect::new(move |_| {
        if let Some(comp) = ctx.editing.get() {
            set_org_id.set(comp.organization_id);
            set_code.set(comp.code);
            set_lookup_id.set(comp.master_lookup_id);
            set_name_th.set(comp.name_th);
            set_name_en.set(comp.name_en);
            set_active.set(comp.active);
        } else {
            if org_id.get_untracked().is_empty() {
                if let Some(default_org) = ctx.default_org_id.get_untracked() {
                    set_org_id.set(default_org);
                } else if let Some(first_org) = orgs.get().first() {
                    set_org_id.set(first_org.id.clone());
                }
            }
            if lookup_id.get_untracked().is_empty() {
                if let Some(first_type) = company_types.get().first() {
                    set_lookup_id.set(first_type.id.clone());
                }
            }
        }
    });

    let on_submit = move |_| {
        let parent_org = org_id.get().trim().to_string();
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();
        let lookup = lookup_id.get().trim().to_string();

        if parent_org.is_empty() {
            set_error_msg.set(Some(t_or("dbpg001.company.validation.parent_org", "กรุณาเลือกองค์กรแม่ (Parent Organization)")));
            return;
        }
        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg001.company.validation.code", "กรุณากรอกรหัสบริษัท (Company Code)")));
            return;
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg001.company.validation.name_th", "กรุณากรอกชื่อบริษัทภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateCompanyInput {
            organization_id: parent_org,
            code: c,
            master_lookup_id: if lookup.is_empty() {
                DEFAULT_COMPANY_TYPE_ID.to_string()
            } else {
                lookup
            },
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
                                <span class="text-sm">{t_or("dbpg001.company.loading", "กำลังโหลดข้อมูลบริษัท...")}</span>
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
                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.company.field.parent_org", "สังกัดกลุ่มองค์กร (Parent Organization) *")}
                        </label>
                        <Select
                            value=Signal::derive(move || org_id.get())
                            on_change=Callback::new(move |val| set_org_id.set(val))
                            class="bg-background text-sm"
                        >
                            {move || {
                                orgs.get().into_iter().map(|org| {
                                    let label = org.display_name();
                                    view! {
                                        <SelectOption value=org.id.clone()>
                                            {label}
                                        </SelectOption>
                                    }
                                }).collect_view()
                            }}
                        </Select>
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.company.field.code", "รหัสบริษัท (Code) *")}
                        </label>
                        <Input
                            value=Signal::derive(move || code.get())
                            on_input=Callback::new(move |val| set_code.set(val))
                            placeholder=t_or("dbpg001.company.field.code_placeholder", "เช่น CMP001")
                            class="font-mono text-sm"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.company.field.type", "ประเภทนิติบุคคล (Company Type)")}
                        </label>
                        <Select
                            value=Signal::derive(move || lookup_id.get())
                            on_change=Callback::new(move |val| set_lookup_id.set(val))
                            class="bg-background text-sm"
                        >
                            {move || {
                                company_types.get().into_iter().map(|t| {
                                    view! {
                                        <SelectOption value=t.id.clone()>
                                            {format!("{} ({})", t.name_th, t.code)}
                                        </SelectOption>
                                    }
                                }).collect_view()
                            }}
                        </Select>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.company.field.name_th", "ชื่อบริษัท (ภาษาไทย) *")}
                        </label>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |val| set_name_th.set(val))
                            placeholder=t_or("dbpg001.company.field.name_th_placeholder", "เช่น บริษัท เอบีซี เทรดดิ้ง จำกัด")
                            class="text-sm"
                        />
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">
                            {t_or("dbpg001.company.field.name_en", "ชื่อบริษัท (English)")}
                        </label>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |val| set_name_en.set(val))
                            placeholder="e.g. ABC Trading Co., Ltd."
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
                            label=t_or("dbpg001.company.delete", "ลบบริษัท")
                            confirm_text=t_or("dbpg001.company.delete_confirm", "ลบบริษัทนี้ถาวร?")
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
