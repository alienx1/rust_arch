//! Password policy: how long a new password must be and what it must contain

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardDescription, CardHeader, CardTitle, Checkbox};

use crate::access::{Action, can};
use crate::pages::su::supg006::components::NumberField;
use crate::pages::su::supg006::models::POLICY_FLAGS;
use crate::pages::su::supg006::providers::use_supg006;

#[component]
pub fn PasswordPolicyCard() -> impl IntoView {
    let ctx = use_supg006();
    let can_write = can("SUPG006", Action::Write);

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardHeader>
                <CardTitle class="text-base">{move || t_or("supg006.policy.title", "นโยบายรหัสผ่าน")}</CardTitle>
                <CardDescription>
                    {move || t_or("supg006.policy.description", "ใช้กับรหัสผ่านใหม่ที่ผู้ใช้ตั้งเอง (จากลิงก์ในอีเมลหรือหน้าเปลี่ยนรหัสผ่าน) ระบบตรวจก่อนส่งให้ Kratos และบอกผู้ใช้ว่าขาดข้อไหน รหัสผ่านเดิมที่ตั้งไว้แล้วไม่ถูกตรวจย้อนหลัง จนกว่าจะเปลี่ยนรหัสผ่านครั้งถัดไป")}
                </CardDescription>
            </CardHeader>
            <CardContent class="space-y-5">
                <fieldset class="contents" disabled=move || !can_write.get()>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                        <NumberField key="security.password.min_length" />
                    </div>
                    <div class="space-y-3 pt-1">
                        <div class="text-xs font-semibold text-muted-foreground">{t_or("supg006.policy.rules", "รหัสผ่านใหม่ต้อง")}</div>
                        {POLICY_FLAGS.iter().map(|(key, fallback, label_key)| {
                            let key = *key;
                            view! {
                                <Checkbox
                                    checked=Signal::derive(move || ctx.value_of(key) == "1")
                                    on_change=Callback::new(move |on: bool| ctx.set_value.run((key.to_string(), if on { "1" } else { "0" }.to_string())))
                                    label=t_or(label_key, fallback)
                                />
                            }
                        }).collect_view()}
                    </div>
                </fieldset>
            </CardContent>
        </Card>
    }
}
