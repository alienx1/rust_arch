//! Preview of a mail body with sample values: HTML in a sandboxed frame (no scripts, no links
//! followed), plain text in a block

use leptos::prelude::*;
use web_package::core::i18n::t_or;

#[component]
pub fn MailPreviewFrame(
    /// HTML with the placeholders already filled in; empty = the built-in text is used
    #[prop(into)]
    html: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="rounded-md border border-border/60 overflow-hidden bg-white">
            {move || if html.get().trim().is_empty() {
                view! {
                    <div class="p-6 text-xs text-muted-foreground text-center">
                        {t_or("supg006.preview.built_in", "ว่างไว้ = ใช้ข้อความมาตรฐานของระบบ (ไทย/อังกฤษ)")}
                    </div>
                }.into_any()
            } else {
                // An empty `sandbox` turns every capability off
                view! { <iframe sandbox="" srcdoc=move || html.get() class="w-full h-[480px] bg-white" title="mail preview"></iframe> }.into_any()
            }}
        </div>
    }
}
