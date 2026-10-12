//! Raw Session Inspector & Developer Diagnostics Component

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{
    Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader, CardTitle,
};

#[component]
pub fn SessionRawInspector(raw_json: String) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let json_text = raw_json.clone();

    let on_copy = move |_| {
        set_copied.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let nav = window.navigator();
                let clipboard = nav.clipboard();
                let _ = clipboard.write_text(&json_text);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &json_text;
    };

    view! {
        <Card class="border-border/60 shadow-sm bg-card">
            <CardHeader class="pb-3 border-b border-border/40">
                <div class="flex items-center justify-between">
                    <div>
                        <CardTitle class="text-base font-semibold flex items-center gap-2">
                            <svg class="w-4 h-4 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <path d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                            </svg>
                            {t_or("supg005.raw.title", "ข้อมูลดิบเซสชัน (Ory Kratos / IAM Session JSON Inspector)")}
                        </CardTitle>
                        <CardDescription>
                            {t_or("supg005.raw.description", "ข้อมูล Payload เชิงเทคนิคจาก /sessions/whoami สำหรับการตรวจสอบความถูกต้อง (Diagnostics & Auditing)")}
                        </CardDescription>
                    </div>

                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-8 gap-1.5"
                        on_click=Callback::new(on_copy)
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                        </svg>
                        {move || if copied.get() { t_or("supg005.raw.copied", "คัดลอก JSON แล้ว ✓") } else { t_or("supg005.raw.copy", "คัดลอก JSON") }}
                    </Button>
                </div>
            </CardHeader>
            <CardContent class="p-4">
                <pre class="bg-muted/70 text-foreground font-mono text-[11px] p-4 rounded-lg overflow-x-auto border border-border/50 max-h-[460px] leading-relaxed">
                    <code>{raw_json}</code>
                </pre>
            </CardContent>
        </Card>
    }
}
