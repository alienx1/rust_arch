//! One number setting: label, input, hint, allowed range and a red note when the value is out of range

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::Input;

use crate::pages::su::supg006::models::number_field_text;
use crate::pages::su::supg006::providers::use_supg006;

#[component]
pub fn NumberField(key: &'static str) -> impl IntoView {
    let ctx = use_supg006();
    let (label, hint) = number_field_text(key);
    let item = move || ctx.item(key);
    // Out of range or not a whole number: a red note under the field; the server checks again on save
    let invalid = move || {
        let Some(i) = item() else { return false };
        match ctx.value_of(key).trim().parse::<i64>() {
            Ok(n) => i.min.is_some_and(|m| n < m) || i.max.is_some_and(|m| n > m),
            Err(_) => true,
        }
    };
    let range = move || {
        item().and_then(|i| Some((i.min?, i.max?, i.default_value))).map(|(min, max, default)| {
            t_fmt(
                "supg006.field.range",
                "ช่วงที่ใช้ได้ {min}–{max} • ค่าเริ่มต้น {default}",
                &[("min", &min.to_string()), ("max", &max.to_string()), ("default", &default)],
            )
        })
    };

    view! {
        <div class="space-y-1.5">
            <label class="text-xs font-semibold text-muted-foreground">
                {move || t_or(&format!("supg006.field.{key}.label"), label)}
            </label>
            <Input
                r#type="number"
                value=Signal::derive(move || ctx.value_of(key))
                on_input=Callback::new(move |v: String| ctx.set_value.run((key.to_string(), v)))
                class="font-mono"
            />
            {move || invalid().then(|| view! {
                <div class="text-[11px] text-destructive font-medium">
                    {t_or("supg006.field.invalid", "ต้องเป็นจำนวนเต็มในช่วงที่กำหนด")}
                </div>
            })}
            <div class="text-[11px] text-muted-foreground">
                {move || t_or(&format!("supg006.field.{key}.hint"), hint)}
                {move || range().map(|r| view! { <span class="block mt-0.5 font-mono">{r}</span> })}
            </div>
        </div>
    }
}

