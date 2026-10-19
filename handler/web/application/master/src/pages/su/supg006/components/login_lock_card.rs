//! Sign-in brute-force lock settings: how many wrong passwords lock an account, for how long

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardDescription, CardHeader, CardTitle};

use crate::access::{Action, can};
use crate::pages::su::supg006::components::NumberField;
use crate::pages::su::supg006::models::ConfigGroup;
use crate::pages::su::supg006::providers::use_supg006;

/// Keys of the number settings, in the order they are shown
const NUMBER_KEYS: [&str; 4] = [
    "security.login.max_failed_attempts",
    "security.login.attempt_window_minutes",
    "security.login.lock_minutes",
    "security.password.mail_resend_minutes",
];

#[component]
pub fn LoginLockCard() -> impl IntoView {
    let can_write = can("SUPG006", Action::Write);
    let ctx = use_supg006();
    let shown = move |group: ConfigGroup| ctx.items.with(|items| items.iter().any(|i| i.group == group));

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardHeader>
                <CardTitle class="text-base">{move || t_or("supg006.login.title", "การป้องกันการเดารหัสผ่าน (Brute-force)")}</CardTitle>
                <CardDescription>
                    {move || t_or("supg006.login.description", "นับครั้งที่ใส่รหัสผ่านผิดต่อชื่อผู้ใช้ เมื่อครบจะล็อกบัญชีชั่วคราว (ล็อกอินไม่ได้แม้ใส่รหัสถูก) ชื่อผู้ใช้ที่ไม่มีอยู่จริงก็นับเช่นเดียวกัน เพื่อไม่ให้บอกได้ว่ามีบัญชีนี้หรือไม่")}
                </CardDescription>
            </CardHeader>
            <CardContent class="space-y-5">
                <fieldset class="contents" disabled=move || !can_write.get()>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                        {move || shown(ConfigGroup::Login).then(|| view! {
                            <NumberField key=NUMBER_KEYS[0] />
                            <NumberField key=NUMBER_KEYS[1] />
                            <NumberField key=NUMBER_KEYS[2] />
                        })}
                        {move || shown(ConfigGroup::Password).then(|| view! { <NumberField key=NUMBER_KEYS[3] /> })}
                    </div>
                </fieldset>
            </CardContent>
        </Card>
    }
}
