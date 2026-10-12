//! Banner shown when the audit store (Quickwit) is unavailable.

use leptos::prelude::*;
use web_package::core::i18n::t_or;

use crate::pages::al::alpg000::providers::use_alpg000;

#[component]
pub fn AuditStatusBanner() -> impl IntoView {
    let ctx = use_alpg000();
    view! {
        {move || ctx.unavailable.get().then(|| view! {
            <div class="p-3.5 rounded-lg bg-warning/10 border border-warning/30 text-xs shadow-sm space-y-1">
                <p class="font-medium text-foreground">
                    {t_or("alpg000.banner.unavailable", "บริการบันทึกเหตุการณ์ (Quickwit) ไม่พร้อมใช้งาน จึงยังไม่มีรายการแสดง")}
                </p>
                {ctx.unavailable_message.get().map(|m| view! {
                    <p class="text-muted-foreground font-mono break-all">{m}</p>
                })}
            </div>
        })}
        {move || ctx.error.get().map(|err| view! {
            <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm">
                <span class="font-medium">{err}</span>
            </div>
        })}
    }
}
