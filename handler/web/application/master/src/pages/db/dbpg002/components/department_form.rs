//! Department Form (Create / Edit Department, DBPG002)
//!
//! Form for `db_department` & `db_department_label`. Organization and company follow the chosen
//! branch on the server, so only the branch is picked here.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{SaveConfirmButton, 
    Button, ButtonSize, ButtonVariant, Card, DeleteConfirmButton, CardContent, Checkbox, Input, Select, SelectOption,
    Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg002::models::CreateDepartmentInput;
use crate::pages::db::dbpg002::providers::use_dbpg002_department_form;

#[component]
pub fn DepartmentForm() -> impl IntoView {
    let ctx = use_dbpg002_department_form();
    let can_write = can("DBPG002", Action::Write);
    let can_approve = can("DBPG002", Action::Approve);

    let (branch_id, set_branch_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    // Empty = no default permission group
    let (default_group, set_default_group) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    Effect::new(move |_| {
        if let Some(d) = ctx.editing.get() {
            set_branch_id.set(d.branch_id);
            set_code.set(d.code);
            set_name_th.set(d.name_th);
            set_name_en.set(d.name_en);
            set_active.set(d.active);
            set_default_group.set(d.default_permission_group_id.unwrap_or_default());
        } else if branch_id.get_untracked().is_empty() {
            if let Some(default_branch) = ctx.default_branch_id.get_untracked() {
                set_branch_id.set(default_branch);
            } else if let Some(first) = ctx.branches.get().first() {
                set_branch_id.set(first.id.clone());
            }
        }
    });

    let on_submit = move |_| {
        let branch = branch_id.get().trim().to_string();
        let c = code.get().trim().to_uppercase();
        let th = name_th.get().trim().to_string();

        if branch.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.dept.error.branch_required", "กรุณาเลือกสาขา (ต้องมีสาขาก่อนจึงจะเพิ่มแผนกได้)")));
            return;
        }
        if c.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.dept.error.code_required", "กรุณากรอกรหัสแผนก (Department Code)")));
            return;
        }
        if th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg002.dept.error.name_th_required", "กรุณากรอกชื่อแผนกภาษาไทย")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(CreateDepartmentInput {
            branch_id: branch,
            code: c,
            active: active.get(),
            name_th: th,
            name_en: name_en.get().trim().to_string(),
            default_permission_group_id: Some(default_group.get()).filter(|g| !g.is_empty()),
        });
    };

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardContent class="p-6 space-y-4">
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("dbpg002.dept.form.loading", "กำลังโหลดข้อมูลแผนก...")}</span>
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
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.dept.form.branch_label", "สังกัดสาขา (Branch) *")}</label>
                        <Select
                            value=Signal::derive(move || branch_id.get())
                            on_change=Callback::new(move |v| set_branch_id.set(v))
                            class="bg-background text-sm"
                        >
                            {move || ctx.branches.get().into_iter().map(|b| {
                                let label = format!("{} • {}", b.company_code, b.display_name());
                                view! { <SelectOption value=b.id.clone()>{label}</SelectOption> }
                            }).collect_view()}
                        </Select>
                    </div>

                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.dept.form.code_label", "รหัสแผนก (Code) *")}</label>
                        <Input
                            value=Signal::derive(move || code.get())
                            on_input=Callback::new(move |v| set_code.set(v))
                            placeholder=t_or("dbpg002.dept.form.code_placeholder", "เช่น IT, HR, ACC")
                            class="font-mono text-sm"
                        />
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.dept.form.name_th_label", "ชื่อแผนก (ภาษาไทย) *")}</label>
                        <Input
                            value=Signal::derive(move || name_th.get())
                            on_input=Callback::new(move |v| set_name_th.set(v))
                            placeholder=t_or("dbpg002.dept.form.name_th_placeholder", "เช่น เทคโนโลยีสารสนเทศ")
                            class="text-sm"
                        />
                    </div>
                    <div class="space-y-1.5">
                        <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.dept.form.name_en_label", "ชื่อแผนก (English)")}</label>
                        <Input
                            value=Signal::derive(move || name_en.get())
                            on_input=Callback::new(move |v| set_name_en.set(v))
                            placeholder="e.g. Information Technology"
                            class="text-sm"
                        />
                    </div>
                </div>

                <div class="space-y-1.5">
                    <label class="text-xs font-semibold text-muted-foreground">{t_or("dbpg002.dept.form.default_group_label", "กลุ่มสิทธิ์เริ่มต้นของแผนก (Default Permission Group)")}</label>
                    <Select
                        value=Signal::derive(move || default_group.get())
                        on_change=Callback::new(move |v| set_default_group.set(v))
                        class="bg-background text-sm"
                    >
                        <SelectOption value="">{t_or("dbpg002.dept.form.default_group_none", "— ไม่กำหนด —")}</SelectOption>
                        {move || {
                            let groups = ctx.permission_groups.get();
                            // Whoever may not list the groups still sees (and keeps) the current default
                            let current = ctx.editing.get().and_then(|d| d.default_permission_group_id.zip(d.default_permission_group_code));
                            let extra = current.filter(|(id, _)| !groups.iter().any(|g| &g.id == id));
                            extra
                                .into_iter()
                                .map(|(id, code)| view! { <SelectOption value=id>{code}</SelectOption> }.into_any())
                                .chain(groups.into_iter().map(|g| view! { <SelectOption value=g.id.clone()>{g.code.clone()}</SelectOption> }.into_any()))
                                .collect_view()
                        }}
                    </Select>
                    <div class="text-[11px] text-muted-foreground">
                        {t_or("dbpg002.dept.form.default_group_hint", "บัญชีของพนักงานในแผนกนี้จะได้กลุ่มสิทธิ์นี้ภายในขอบเขตสาขาของแผนกโดยอัตโนมัติ ทั้งบัญชีที่สร้างใหม่และบัญชีที่มีอยู่ (เพิ่มให้ ไม่ลบสิทธิ์เดิม และแก้ไขได้ที่หน้าบัญชีผู้ใช้) กลุ่ม ADMIN ใช้เป็นค่าเริ่มต้นไม่ได้")}
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
                    <Show when=move || ctx.editing.with(|e| e.is_some())>
                        <DeleteConfirmButton
                            label=t_or("dbpg002.dept.form.delete", "ลบแผนก")
                            confirm_text=t_or("dbpg002.dept.form.delete_confirm", "ลบแผนกนี้ถาวร?")
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
                        <SaveConfirmButton
                            label=Signal::derive(move || if ctx.is_saving.get() {
                                t_or("dbpg002.form.saving", "กำลังบันทึก...")
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
