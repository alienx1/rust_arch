//! Mail templates: pick one of the four password mails, edit its subject and bodies, see the
//! HTML with sample values

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Textarea};

use crate::access::{Action, can};
use crate::pages::su::supg006::components::MailPreviewFrame;
use crate::pages::su::supg006::models::{APP_NAME_KEY, MAIL_KINDS, PLACEHOLDERS, render_preview, template_key};
use crate::pages::su::supg006::providers::use_supg006;

#[component]
pub fn MailTemplateEditor() -> impl IntoView {
    let ctx = use_supg006();
    let can_write = can("SUPG006", Action::Write);
    let (kind, set_kind) = signal(MAIL_KINDS[0].code.to_string());

    let key = move |part: &str| template_key(&kind.get(), part);
    let html_preview = Signal::derive(move || render_preview(&ctx.value_of(&key("html")), &ctx.value_of(APP_NAME_KEY)));
    let text_preview = Signal::derive(move || render_preview(&ctx.value_of(&key("text")), &ctx.value_of(APP_NAME_KEY)));

    view! {
        <Card class="border-border/60 bg-card shadow-sm">
            <CardHeader>
                <CardTitle class="text-base">{move || t_or("supg006.mail.title", "เทมเพลตอีเมลตั้งรหัสผ่าน")}</CardTitle>
                <CardDescription>
                    {move || t_or("supg006.mail.description", "ข้อความที่ระบบส่งให้ผู้ใช้ ช่องที่ว่างไว้จะใช้ข้อความมาตรฐานของระบบ เนื้อหา (HTML และข้อความล้วน) ต้องมี {{link}} ไม่เช่นนั้นผู้ใช้จะตั้งรหัสผ่านไม่ได้")}
                </CardDescription>
            </CardHeader>
            <CardContent class="space-y-5">
                <div class="flex flex-wrap gap-2" role="tablist">
                    {MAIL_KINDS.iter().map(|k| {
                        let code = k.code;
                        view! {
                            <button
                                type="button"
                                role="tab"
                                class=move || if kind.get() == code {
                                    "px-3 py-1.5 rounded-md text-xs font-medium bg-primary text-primary-foreground cursor-pointer"
                                } else {
                                    "px-3 py-1.5 rounded-md text-xs font-medium border border-border text-muted-foreground hover:bg-muted cursor-pointer"
                                }
                                on:click=move |_| set_kind.set(code.to_string())
                            >
                                {move || t_or(k.label_key, k.label_fallback)}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <div class="text-[11px] text-muted-foreground">
                    {move || {
                        let current = kind.get();
                        MAIL_KINDS.iter().find(|k| k.code == current).map(|k| t_or(k.hint_key, k.hint_fallback)).unwrap_or_default()
                    }}
                </div>

                <div class="grid grid-cols-1 xl:grid-cols-2 gap-5">
                    <fieldset class="contents" disabled=move || !can_write.get()>
                        <div class="space-y-4">
                            <div class="space-y-1.5">
                                <label class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.app_name", "ชื่อระบบในหัวอีเมล (ใช้กับทุกชุด)")}</label>
                                <Input
                                    value=Signal::derive(move || ctx.value_of(APP_NAME_KEY))
                                    on_input=Callback::new(move |v: String| ctx.set_value.run((APP_NAME_KEY.to_string(), v)))
                                    placeholder="Arch"
                                />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.subject", "หัวเรื่อง")}</label>
                                <Input
                                    value=Signal::derive(move || ctx.value_of(&key("subject")))
                                    on_input=Callback::new(move |v: String| ctx.set_value.run((key("subject"), v)))
                                />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.html", "เนื้อหา HTML")}</label>
                                <Textarea
                                    rows=14
                                    class="font-mono text-xs"
                                    value=Signal::derive(move || ctx.value_of(&key("html")))
                                    on_input=Callback::new(move |v: String| ctx.set_value.run((key("html"), v)))
                                />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.text", "เนื้อหาข้อความล้วน (สำหรับโปรแกรมอีเมลที่ไม่แสดง HTML)")}</label>
                                <Textarea
                                    rows=8
                                    class="font-mono text-xs"
                                    value=Signal::derive(move || ctx.value_of(&key("text")))
                                    on_input=Callback::new(move |v: String| ctx.set_value.run((key("text"), v)))
                                />
                            </div>
                            <div class="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
                                <span>{t_or("supg006.mail.placeholders", "ตัวแปรที่ใช้ได้:")}</span>
                                {PLACEHOLDERS.iter().map(|p| view! {
                                    <code class="px-1.5 py-0.5 rounded bg-muted font-mono">{format!("{{{{{p}}}}}")}</code>
                                }).collect_view()}
                            </div>
                        </div>
                    </fieldset>

                    <div class="space-y-3">
                        <div class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.preview", "ตัวอย่าง (ข้อมูลสมมติ)")}</div>
                        <MailPreviewFrame html=html_preview />
                        <div class="text-xs font-semibold text-muted-foreground">{t_or("supg006.mail.preview_text", "ตัวอย่างข้อความล้วน")}</div>
                        <pre class="p-3 rounded-md border border-border/60 bg-muted/30 text-[11px] font-mono whitespace-pre-wrap break-words max-h-48 overflow-auto">
                            {move || if text_preview.get().trim().is_empty() { t_or("supg006.preview.built_in", "ว่างไว้ = ใช้ข้อความมาตรฐานของระบบ (ไทย/อังกฤษ)") } else { text_preview.get() }}
                        </pre>
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
