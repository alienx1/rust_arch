//! Assignment editor (SUPG004): the groups an account holds, each at one scope unit.
//!
//! The list shows `<group> @ <level>: <path — name>` with a remove button. A new row is added by
//! picking a group, a scope level and the unit down the chain organization → company → branch →
//! department (a step offers only the children of the one above). The first row is the primary one.

use leptos::prelude::*;
use web_package::core::i18n::{localized, t_fmt, t_or};
use web_package::ui::component::{Button, ButtonSize, ButtonVariant, Select, SelectOption};

use crate::access::{Action, can};
use crate::pages::su::supg004::models::{
    SCOPE_LEVELS, ScopeOptionDto, UserAssignmentDto, scope_level_index, scope_level_label,
};
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

/// One step of the org → company → branch → department chain
#[component]
fn ScopeStep(
    #[prop(into)] label: String,
    #[prop(into)] value: Signal<String>,
    on_change: Callback<String>,
    #[prop(into)] options: Signal<Vec<ScopeOptionDto>>,
) -> impl IntoView {
    view! {
        <Field label=label>
            <Select value=value on_change=on_change class="bg-background text-sm">
                {move || options.get().into_iter().map(|o| {
                    let label = o.display_name();
                    view! { <SelectOption value=o.id.clone()>{label}</SelectOption> }
                }).collect_view()}
            </Select>
        </Field>
    }
}

/// Keeps a selection valid for its parent (first child, or empty when there is none)
fn keep_valid(options: Vec<ScopeOptionDto>, current: &str, set: WriteSignal<String>) {
    if !options.iter().any(|o| o.id == current) {
        set.set(options.first().map(|o| o.id.clone()).unwrap_or_default());
    }
}

#[component]
pub fn AssignmentEditor(
    assignments: ReadSignal<Vec<UserAssignmentDto>>,
    set_assignments: WriteSignal<Vec<UserAssignmentDto>>,
    /// Shows a validation message in the form
    on_error: Callback<Option<String>>,
) -> impl IntoView {
    let ctx = use_supg004_form();
    let can_approve = can("SUPG004", Action::Approve);

    let (sel_group, set_sel_group) = signal(String::new());
    let (scope_level, set_scope_level) = signal("organization".to_string());
    let (sel_org, set_sel_org) = signal(String::new());
    let (sel_company, set_sel_company) = signal(String::new());
    let (sel_branch, set_sel_branch) = signal(String::new());
    let (sel_dept, set_sel_dept) = signal(String::new());

    // Preselect the first group the caller may assign
    Effect::new(move |_| {
        let groups = ctx.groups.get();
        let approver = can_approve.get();
        if !groups.iter().any(|g| g.id == sel_group.get_untracked()) {
            set_sel_group.set(
                groups.iter().find(|g| g.active && (approver || !g.needs_approval)).map(|g| g.id.clone()).unwrap_or_default(),
            );
        }
    });

    let org_options = Signal::derive(move || ctx.scope_options.get().children("organization", ""));
    let company_options = Signal::derive(move || ctx.scope_options.get().children("company", &sel_org.get()));
    let branch_options = Signal::derive(move || ctx.scope_options.get().children("branch", &sel_company.get()));
    let dept_options = Signal::derive(move || ctx.scope_options.get().children("department", &sel_branch.get()));
    Effect::new(move |_| keep_valid(org_options.get(), &sel_org.get_untracked(), set_sel_org));
    Effect::new(move |_| keep_valid(company_options.get(), &sel_company.get_untracked(), set_sel_company));
    Effect::new(move |_| keep_valid(branch_options.get(), &sel_branch.get_untracked(), set_sel_branch));
    Effect::new(move |_| keep_valid(dept_options.get(), &sel_dept.get_untracked(), set_sel_dept));
    let depth = Signal::derive(move || scope_level_index(&scope_level.get()));

    let needs_approval = Signal::derive(move || {
        ctx.groups.get().iter().any(|g| g.id == sel_group.get() && g.needs_approval)
    });

    let add_assignment = move |_| {
        let level = scope_level.get();
        let group_id = sel_group.get();
        let Some(group) = ctx.groups.get().into_iter().find(|g| g.id == group_id) else {
            on_error.run(Some(t_or("supg004.form.err_group", "กรุณาเลือกกลุ่มสิทธิ์")));
            return;
        };
        let (id, options) = match depth.get() {
            0 => (sel_org.get(), org_options.get()),
            1 => (sel_company.get(), company_options.get()),
            2 => (sel_branch.get(), branch_options.get()),
            _ => (sel_dept.get(), dept_options.get()),
        };
        let Some(option) = options.into_iter().find(|o| o.id == id) else {
            on_error.run(Some(t_or("supg004.form.err_no_options", "ยังไม่มีข้อมูลในระดับนี้ให้เลือก")));
            return;
        };
        if assignments.get().iter().any(|a| a.group_id == group.id && a.scope_id == id) {
            on_error.run(Some(t_or("supg004.form.err_assignment_duplicate", "เพิ่มกลุ่มสิทธิ์นี้ในขอบเขตนี้ไปแล้ว")));
            return;
        }
        // Codes from the organization down to the picked unit
        let all = ctx.scope_options.get();
        let mut chain = vec![option.code.clone()];
        let mut parent = option.parent_id.clone();
        for up in SCOPE_LEVELS.iter().take(depth.get()).rev() {
            let Some(p) = parent.take() else { break };
            if let Some(o) = all.for_level(up).into_iter().find(|o| o.id == p) {
                chain.push(o.code.clone());
                parent = o.parent_id.clone();
            }
        }
        chain.reverse();
        on_error.run(None);
        set_assignments.update(|list| {
            list.push(UserAssignmentDto {
                group_id: group.id.clone(),
                group_code: group.code.clone(),
                scope_level: level,
                scope_id: id,
                scope_code: option.code.clone(),
                scope_name: localized(&option.name_th, &option.name_en),
                scope_path: chain.join(" › "),
            })
        });
    };

    view! {
        <div class="space-y-2">
            <div class="text-xs font-semibold text-muted-foreground">
                {move || t_fmt("supg004.form.assignments_heading", "กลุ่มสิทธิ์และขอบเขต ({count} รายการ) *", &[("count", &assignments.get().len().to_string())])}
            </div>
            {move || assignments.get().is_empty().then(|| view! {
                <div class="p-3 text-xs rounded-md border border-dashed border-border text-muted-foreground">
                    {t_or("supg004.form.assignments_empty", "ยังไม่มีรายการ — เลือกกลุ่มสิทธิ์และขอบเขตด้านล่างแล้วกด “เพิ่ม” (เพิ่มได้หลายรายการ)")}
                </div>
            })}
            <div class="flex flex-col gap-1.5">
                {move || assignments.get().into_iter().enumerate().map(|(idx, a)| {
                    let label = a.display_name();
                    view! {
                        <div class="flex items-center justify-between gap-2 px-3 py-1.5 rounded-md border border-border/60 bg-muted/20 text-xs">
                            <div class="min-w-0">
                                <span>{label}</span>
                                {(idx == 0).then(|| view! { <span class="ml-2 text-[10px] text-muted-foreground">{t_or("supg004.form.primary", "(หลัก)")}</span> })}
                            </div>
                            <button
                                type="button"
                                class="text-destructive hover:underline shrink-0"
                                on:click=move |_| set_assignments.update(|list| { if idx < list.len() { list.remove(idx); } })
                            >
                                {t_or("supg004.form.remove", "ลบ")}
                            </button>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>

        <div class="space-y-3 p-3 rounded-md border border-border/60 bg-muted/10">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <Field label=t_or("supg004.form.group", "กลุ่มสิทธิ์ (Permission group) *")>
                    <Select
                        value=Signal::derive(move || sel_group.get())
                        on_change=Callback::new(move |v| set_sel_group.set(v))
                        class="bg-background text-sm"
                    >
                        {move || ctx.groups.get().into_iter().filter(|g| g.active).map(|g| {
                            // ADMIN-type groups can be assigned only with APPROVE on SUPG004
                            let locked = g.needs_approval && !can_approve.get();
                            let label = if g.needs_approval { format!("{} 🔒", g.display_name()) } else { g.display_name() };
                            view! { <SelectOption value=g.id.clone() disabled=locked>{label}</SelectOption> }
                        }).collect_view()}
                    </Select>
                </Field>
                <Field label=t_or("supg004.form.scope_level", "ระดับขอบเขต (Scope level)")>
                    <Select
                        value=Signal::derive(move || scope_level.get())
                        on_change=Callback::new(move |v| set_scope_level.set(v))
                        class="bg-background text-sm"
                    >
                        {SCOPE_LEVELS.iter().map(|k| view! {
                            <SelectOption value=k.to_string()>{scope_level_label(k)}</SelectOption>
                        }).collect_view()}
                    </Select>
                </Field>
                <ScopeStep
                    label=t_or("supg004.scope.organization", "องค์กร (Organization)")
                    value=sel_org
                    on_change=Callback::new(move |v| set_sel_org.set(v))
                    options=org_options
                />
                {move || (depth.get() >= 1).then(|| view! {
                    <ScopeStep
                        label=t_or("supg004.scope.company", "บริษัท (Company)")
                        value=sel_company
                        on_change=Callback::new(move |v| set_sel_company.set(v))
                        options=company_options
                    />
                })}
                {move || (depth.get() >= 2).then(|| view! {
                    <ScopeStep
                        label=t_or("supg004.scope.branch", "สาขา (Branch)")
                        value=sel_branch
                        on_change=Callback::new(move |v| set_sel_branch.set(v))
                        options=branch_options
                    />
                })}
                {move || (depth.get() >= 3).then(|| view! {
                    <ScopeStep
                        label=t_or("supg004.scope.department", "แผนก (Department)")
                        value=sel_dept
                        on_change=Callback::new(move |v| set_sel_dept.set(v))
                        options=dept_options
                    />
                })}
            </div>
            {move || needs_approval.get().then(|| view! {
                <div class="p-3 text-xs rounded-md bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                    {t_or("supg004.form.admin_group_notice", "🔒 กลุ่มผู้ดูแลระบบ: ต้องมีสิทธิ์ APPROVE ในโปรแกรมนี้จึงจะกำหนดได้")}
                </div>
            })}
            <div class="flex justify-end">
                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(add_assignment)>
                    {move || t_fmt("supg004.form.add_assignment", "+ เพิ่ม ({level})", &[("level", &scope_level_label(&scope_level.get()).to_string())])}
                </Button>
            </div>
        </div>
    }
}
