//! Geography Form (Create / Edit province, district, sub-district or postal code, DBPG005)
//!
//! Province → district → sub-district are cascading selectors loaded from the API. Code and
//! parents are chosen on create and fixed afterwards (addresses and the levels below reference
//! them); editing changes names / postal code value and status.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{
    Button, ButtonSize, ButtonVariant, Card, CardContent, Checkbox, DeleteConfirmButton, Input,
    Select, SelectOption, Spinner,
};

use crate::access::{Action, PermissionGate, can};
use crate::pages::db::dbpg005::models::{GeoFormInput, GeoLevel};
use crate::pages::db::dbpg005::providers::use_dbpg005_form;

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
pub fn GeoForm() -> impl IntoView {
    let ctx = use_dbpg005_form();
    let level = ctx.level;
    let can_write = can("DBPG005", Action::Write);

    let (province_id, set_province_id) = signal(String::new());
    let (district_id, set_district_id) = signal(String::new());
    let (sub_district_id, set_sub_district_id) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let (postal_code, set_postal_code) = signal(String::new());
    let (name_th, set_name_th) = signal(String::new());
    let (name_en, set_name_en) = signal(String::new());
    let (active, set_active) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let is_edit = Signal::derive(move || {
        ctx.editing_node.get().is_some() || ctx.editing_postal.get().is_some()
    });
    let needs_province = matches!(level, GeoLevel::District | GeoLevel::SubDistrict | GeoLevel::PostalCode);
    let needs_district = matches!(level, GeoLevel::SubDistrict | GeoLevel::PostalCode);
    let has_names = level != GeoLevel::PostalCode;

    // Seed the fields from the record to edit
    Effect::new(move |_| {
        if let Some(n) = ctx.editing_node.get() {
            set_code.set(n.code.clone());
            set_name_th.set(n.name_th.clone());
            set_name_en.set(n.name_en.clone());
            set_active.set(n.active);
        }
        if let Some(p) = ctx.editing_postal.get() {
            set_postal_code.set(p.postal_code.clone());
            set_active.set(p.active);
        }
    });

    // Defaults when creating: query parents, else the first province
    Effect::new(move |_| {
        if is_edit.get() || !needs_province || !province_id.get_untracked().is_empty() {
            return;
        }
        let provinces = ctx.provinces.get();
        let chosen = ctx
            .default_province_id
            .get_untracked()
            .filter(|p| provinces.iter().any(|x| &x.id == p))
            .or_else(|| provinces.first().map(|p| p.id.clone()));
        if let Some(p) = chosen {
            set_province_id.set(p.clone());
            ctx.load_districts.run(p);
            if let Some(d) = ctx.default_district_id.get_untracked() {
                set_district_id.set(d.clone());
                if needs_district {
                    ctx.load_sub_districts.run(d);
                }
            }
        }
    });

    // Without a district chosen yet, take the first one that loads (sub-district / postal code)
    Effect::new(move |_| {
        if is_edit.get() || !needs_district || !district_id.get_untracked().is_empty() {
            return;
        }
        if let Some(first) = ctx.districts.get().first() {
            set_district_id.set(first.id.clone());
            ctx.load_sub_districts.run(first.id.clone());
        }
    });

    let on_province = Callback::new(move |v: String| {
        set_province_id.set(v.clone());
        set_district_id.set(String::new());
        set_sub_district_id.set(String::new());
        ctx.load_sub_districts.run(String::new());
        ctx.load_districts.run(v);
    });
    let on_district = Callback::new(move |v: String| {
        set_district_id.set(v.clone());
        set_sub_district_id.set(String::new());
        ctx.load_sub_districts.run(v);
    });

    let on_submit = move |_| {
        let c = code.get().trim().to_string();
        let th = name_th.get().trim().to_string();
        let postal = postal_code.get().trim().to_string();

        if !is_edit.get() {
            if needs_province && province_id.get().is_empty() {
                set_error_msg.set(Some(t_or("dbpg005.form.err_province_required", "กรุณาเลือกจังหวัด")));
                return;
            }
            if level == GeoLevel::District && province_id.get().is_empty() {
                set_error_msg.set(Some(t_or("dbpg005.form.err_province_required", "กรุณาเลือกจังหวัด")));
                return;
            }
            if level == GeoLevel::SubDistrict && district_id.get().is_empty() {
                set_error_msg.set(Some(t_or("dbpg005.form.err_district_required", "กรุณาเลือกอำเภอ / เขต")));
                return;
            }
            if has_names && c.is_empty() {
                set_error_msg.set(Some(t_or("dbpg005.form.err_code_required", "กรุณากรอกรหัส (Code)")));
                return;
            }
        }
        if has_names && th.is_empty() {
            set_error_msg.set(Some(t_or("dbpg005.form.err_name_th_required", "กรุณากรอกชื่อภาษาไทย")));
            return;
        }
        if !has_names && postal.is_empty() {
            set_error_msg.set(Some(t_or("dbpg005.form.err_postal_required", "กรุณากรอกรหัสไปรษณีย์")));
            return;
        }
        set_error_msg.set(None);

        ctx.save.run(GeoFormInput {
            province_id: province_id.get(),
            district_id: district_id.get(),
            sub_district_id: sub_district_id.get(),
            code: c,
            postal_code: postal,
            active: active.get(),
            name_th: th,
            name_en: name_en.get().trim().to_string(),
        });
    };

    // Area shown (read-only) when editing a record that has parents
    let area_text = move || {
        if let Some(n) = ctx.editing_node.get() {
            [n.province_name, n.district_name]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" › ")
        } else if let Some(p) = ctx.editing_postal.get() {
            [p.province_name, p.district_name, p.sub_district_name]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" › ")
        } else {
            String::new()
        }
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

                <fieldset class="contents" disabled=move || !can_write.get()>
                // Area: selectors when creating, read-only path when editing
                {(needs_province || level != GeoLevel::Province).then(|| view! {
                    <div class="space-y-3">
                        <h3 class="text-sm font-semibold text-foreground">{t_or("dbpg005.form.area", "พื้นที่ (Area)")}</h3>
                        {move || if is_edit.get() {
                            view! { <ReadOnlyValue value=Signal::derive(area_text) /> }.into_any()
                        } else {
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                                    <Field label=t_or("dbpg005.form.province", "จังหวัด (Province) *")>
                                        <Select
                                            value=Signal::derive(move || province_id.get())
                                            on_change=on_province
                                            class="bg-background text-sm"
                                        >
                                            {move || ctx.provinces.get().into_iter().map(|p| {
                                                let label = p.display_name();
                                                view! { <SelectOption value=p.id.clone()>{label}</SelectOption> }
                                            }).collect_view()}
                                        </Select>
                                    </Field>
                                    {needs_district.then(|| view! {
                                        <Field label=t_or("dbpg005.form.district", "อำเภอ / เขต (District)")>
                                            <Select
                                                value=Signal::derive(move || district_id.get())
                                                on_change=on_district
                                                class="bg-background text-sm"
                                            >
                                                {(level == GeoLevel::PostalCode).then(|| view! {
                                                    <SelectOption value=String::new()>{t_or("dbpg005.form.unspecified", "— ไม่ระบุ —")}</SelectOption>
                                                })}
                                                {move || ctx.districts.get().into_iter().map(|d| {
                                                    let label = d.display_name();
                                                    view! { <SelectOption value=d.id.clone()>{label}</SelectOption> }
                                                }).collect_view()}
                                            </Select>
                                        </Field>
                                    })}
                                    {(level == GeoLevel::PostalCode).then(|| view! {
                                        <Field label=t_or("dbpg005.form.sub_district", "ตำบล / แขวง (Sub-district)")>
                                            <Select
                                                value=Signal::derive(move || sub_district_id.get())
                                                on_change=Callback::new(move |v| set_sub_district_id.set(v))
                                                class="bg-background text-sm"
                                            >
                                                <SelectOption value=String::new()>{t_or("dbpg005.form.unspecified", "— ไม่ระบุ —")}</SelectOption>
                                                {move || ctx.sub_districts.get().into_iter().map(|s| {
                                                    let label = s.display_name();
                                                    view! { <SelectOption value=s.id.clone()>{label}</SelectOption> }
                                                }).collect_view()}
                                            </Select>
                                        </Field>
                                    })}
                                </div>
                            }.into_any()
                        }}
                    </div>
                })}

                <div class="space-y-3">
                    <h3 class="text-sm font-semibold text-foreground">{level.label()}</h3>
                    {has_names.then(|| view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                            <Field label=t_or("dbpg005.form.code", "รหัส (Code) *")>
                                {move || if is_edit.get() {
                                    view! { <ReadOnlyValue value=Signal::derive(move || code.get()) /> }.into_any()
                                } else {
                                    view! {
                                        <Input
                                            value=Signal::derive(move || code.get())
                                            on_input=Callback::new(move |v| set_code.set(v))
                                            placeholder=t_or("dbpg005.form.code_placeholder", "เช่น 10, 1001, 100101")
                                            class="font-mono text-sm"
                                        />
                                    }.into_any()
                                }}
                            </Field>
                            <Field label=t_or("dbpg005.form.name_th", "ชื่อ (ภาษาไทย) *")>
                                <Input
                                    value=Signal::derive(move || name_th.get())
                                    on_input=Callback::new(move |v| set_name_th.set(v))
                                    placeholder=t_or("dbpg005.form.name_th_placeholder", "เช่น กรุงเทพมหานคร")
                                    class="text-sm"
                                />
                            </Field>
                            <Field label=t_or("dbpg005.form.name_en", "ชื่อ (English)")>
                                <Input
                                    value=Signal::derive(move || name_en.get())
                                    on_input=Callback::new(move |v| set_name_en.set(v))
                                    placeholder="e.g. Bangkok"
                                    class="text-sm"
                                />
                            </Field>
                        </div>
                    })}
                    {(!has_names).then(|| view! {
                        <div class="max-w-xs">
                            <Field label=t_or("dbpg005.form.postal_code", "รหัสไปรษณีย์ (Postal code) *")>
                                <Input
                                    value=Signal::derive(move || postal_code.get())
                                    on_input=Callback::new(move |v| set_postal_code.set(v))
                                    placeholder=t_or("dbpg005.form.postal_placeholder", "เช่น 10110")
                                    class="font-mono text-sm"
                                />
                            </Field>
                        </div>
                    })}
                </div>

                <div class="pt-2">
                    <Checkbox
                        checked=Signal::derive(move || active.get())
                        on_change=Callback::new(move |v| set_active.set(v))
                        label=t_or("dbpg005.form.active", "เปิดใช้งานในระบบ (Active)")
                    />
                </div>

                </fieldset>

                <div class="flex items-center justify-between gap-2 pt-2">
                    <div>
                        <PermissionGate program="DBPG005" action=Action::Approve>
                        {move || is_edit.get().then(|| view! {
                            <DeleteConfirmButton
                                label=t_or("dbpg005.form.delete", "ลบข้อมูล")
                                confirm_text=t_or("dbpg005.form.delete_confirm", "ลบรายการนี้ถาวรหรือไม่?")
                                on_confirm=Callback::new(move |_| ctx.delete.run(()))
                                busy=Signal::derive(move || ctx.is_deleting.get() || ctx.is_saving.get())
                            />
                        })}
                        </PermissionGate>
                    </div>
                    <div class="flex gap-2">
                        <Button
                            variant=ButtonVariant::Outline
                            size=ButtonSize::Sm
                            on_click=Callback::new(move |_| ctx.cancel.run(()))
                        >
                            {t_or("common.action.cancel", "ยกเลิก")}
                        </Button>
                        <PermissionGate program="DBPG005" action=Action::Write>
                        <Button
                            variant=ButtonVariant::Default
                            size=ButtonSize::Sm
                            on_click=Callback::new(on_submit)
                        >
                            {move || if ctx.is_saving.get() { t_or("dbpg005.form.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") }}
                        </Button>
                        </PermissionGate>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
