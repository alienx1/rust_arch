//! Outcome of a password mail: where it went, or why it could not be sent together with the
//! one-time link the administrator then hands over by other means.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::ui::component::{Button, ButtonSize, ButtonVariant};

use crate::pages::su::supg004::models::PasswordMailResultDto;

#[component]
pub fn PasswordMailPanel(
    #[prop(into)] username: String,
    result: PasswordMailResultDto,
    #[prop(optional, into)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let link = result.recovery_link.clone();
    let to_copy = link.clone();
    let on_copy = move |_| {
        set_copied.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            if let (Some(window), Some(url)) = (web_sys::window(), to_copy.as_deref()) {
                let _ = window.navigator().clipboard().write_text(url);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &to_copy;
    };

    let (tone, heading) = if result.sent {
        (
            "border-emerald-500/30 bg-emerald-500/10",
            t_fmt(
                "supg004.mail.sent",
                "ส่งอีเมลตั้งรหัสผ่านของ {username} เข้าคิวส่งไปที่ {to} แล้ว (ถ้าส่งไม่สำเร็จ ดูที่ log ของ api-notification)",
                &[("username", &username), ("to", &result.sent_to.clone().unwrap_or_default())],
            ),
        )
    } else {
        (
            "border-amber-500/30 bg-amber-500/10",
            t_fmt(
                "supg004.mail.failed",
                "ส่งอีเมลของ {username} ไม่สำเร็จ ({error}) — ส่งลิงก์ด้านล่างให้ผู้ใช้ด้วยช่องทางอื่น",
                &[("username", &username), ("error", &result.mail_error.clone().unwrap_or_default())],
            ),
        )
    };

    view! {
        <div class=format!("p-4 rounded-lg border space-y-3 {tone}")>
            <div class="text-sm font-semibold text-foreground">{heading}</div>
            {link.clone().map(|url| view! {
                <div class="text-xs text-muted-foreground">
                    {t_or("supg004.mail.link_hint", "ลิงก์ใช้ได้ครั้งเดียวและจะไม่แสดงซ้ำ")}
                </div>
                <div class="px-3 py-2 rounded-md border border-border/60 bg-background font-mono text-xs break-all select-all">{url}</div>
            })}
            <div class="flex justify-end gap-2">
                {link.is_some().then(|| view! {
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm on_click=Callback::new(on_copy.clone())>
                        {move || if copied.get() { t_or("supg004.recovery.copied", "คัดลอกแล้ว ✓") } else { t_or("supg004.recovery.copy", "คัดลอกลิงก์") }}
                    </Button>
                })}
                {on_close.map(|close| view! {
                    <Button variant=ButtonVariant::Ghost size=ButtonSize::Sm on_click=Callback::new(move |_| close.run(()))>
                        {t_or("supg004.recovery.close", "ปิด")}
                    </Button>
                })}
            </div>
        </div>
    }
}
