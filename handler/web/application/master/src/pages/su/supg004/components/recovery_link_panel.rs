//! One-time password-setting link: shown once after creating an account or issuing a new link.

use web_package::core::i18n::{t_or, t_fmt};
use leptos::prelude::*;
use web_package::ui::component::{Button, ButtonSize, ButtonVariant};

use crate::pages::su::supg004::models::RecoveryLinkDto;

#[component]
pub fn RecoveryLinkPanel(
    #[prop(into)] username: String,
    link: RecoveryLinkDto,
    #[prop(optional, into)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let url = link.url.clone();
    let on_copy = move |_| {
        set_copied.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&url);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &url;
    };

    view! {
        <div class="p-4 rounded-lg border border-primary/30 bg-primary/5 space-y-3">
            <div class="text-sm font-semibold text-foreground">
                {t_fmt("supg004.recovery.title", "ลิงก์ตั้งรหัสผ่านของ {username}", &[("username", &username.to_string())])}
            </div>
            <div class="text-xs text-muted-foreground">
                {t_or("supg004.recovery.description", "ส่งลิงก์นี้ให้ผู้ใช้เพื่อตั้งรหัสผ่านครั้งแรก ใช้ได้ครั้งเดียวและจะไม่แสดงซ้ำ")}
                {link.expires_at.clone().map(|t| t_fmt("supg004.recovery.expires", " • หมดอายุ: {time}", &[("time", &t.to_string())]))}
            </div>
            <div class="px-3 py-2 rounded-md border border-border/60 bg-background font-mono text-xs break-all select-all">
                {link.url.clone()}
            </div>
            <div class="flex justify-end gap-2">
                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(on_copy)>
                    {move || if copied.get() { t_or("supg004.recovery.copied", "คัดลอกแล้ว ✓") } else { t_or("supg004.recovery.copy", "คัดลอกลิงก์") }}
                </Button>
                {on_close.map(|close| view! {
                    <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=Callback::new(move |_| close.run(()))>
                        {t_or("supg004.recovery.close", "ปิด")}
                    </Button>
                })}
            </div>
        </div>
    }
}
