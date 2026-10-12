//! Ory Kratos self-service settings page: choose a new password.
//!
//! A recovery link (e.g. the one SUPG004 issues for a new account) signs the user in and sends
//! them here with `?flow=<id>` (`selfservice.flows.settings.ui_url`).

use super::context::use_auth;
use super::login_page::{get_query_param, is_spa_route};
use super::types::KratosLoginFlow;
use crate::ui::component::{
    Alert, AlertDescription, AlertTitle, AlertVariant, Button, ButtonVariant, Card, CardContent,
    CardDescription, CardHeader, CardTitle, Label, Spinner,
};
use crate::core::i18n::{LanguageSwitcher, t_fmt, t_or};
use leptos::prelude::*;

const INPUT_CLASS: &str = "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

#[component]
pub fn SettingsPage(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] default_redirect: Option<String>,
) -> impl IntoView {
    let auth = use_auth();
    let client = auth.client();
    let navigate = leptos_router::hooks::use_navigate();
    let page_title = title.unwrap_or_else(|| t_or("settings.form.password", "ตั้งรหัสผ่านใหม่"));
    let redirect = default_redirect.unwrap_or_else(|| "/".to_string());

    let (flow, set_flow) = signal::<Option<KratosLoginFlow>>(None);
    let (is_loading, set_is_loading) = signal(true);
    let (is_submitting, set_is_submitting) = signal(false);
    let (password, set_password) = signal(String::new());
    let (confirm, set_confirm) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (field_error, set_field_error) = signal(Option::<String>::None);

    Effect::new(move |_| {
        set_is_loading.set(true);
        leptos::task::spawn_local(async move {
            let result = match get_query_param("flow") {
                Some(id) => client.get_settings_flow(&id).await,
                None => client.init_settings_flow().await,
            };
            match result {
                Ok(f) => {
                    let messages = f.general_messages();
                    if !messages.is_empty() {
                        set_error.set(Some(messages.join(" ")));
                    }
                    set_flow.set(Some(f));
                }
                Err(e) => set_error.set(Some(t_fmt(
                    "common.auth.link_expired",
                    "ลิงก์หมดอายุหรือถูกใช้แล้ว กรุณาขอลิงก์ใหม่จากผู้ดูแลระบบ ({error})",
                    &[("error", &e.to_string())],
                ))),
            }
            set_is_loading.set(false);
        });
    });

    let submit = move || {
        let new_password = password.get();
        if new_password.chars().count() < 8 {
            set_field_error.set(Some(t_or("common.auth.password_min", "รหัสผ่านต้องมีอย่างน้อย 8 ตัวอักษร")));
            return;
        }
        if new_password != confirm.get() {
            set_field_error.set(Some(t_or("common.auth.password_mismatch", "รหัสผ่านและการยืนยันไม่ตรงกัน")));
            return;
        }
        let Some(current) = flow.get() else {
            set_error.set(Some(t_or("common.auth.flow_missing", "ยังไม่พบขั้นตอนตั้งรหัสผ่าน กรุณาเปิดลิงก์อีกครั้ง")));
            return;
        };
        set_field_error.set(None);
        set_error.set(None);
        set_is_submitting.set(true);
        let (action, csrf) = (current.ui.action.clone(), current.csrf_token());
        let navigate = navigate.clone();
        let redirect = redirect.clone();
        leptos::task::spawn_local(async move {
            match client.submit_settings_password(&action, &new_password, csrf).await {
                Ok(()) => {
                    if let Ok(Some(session)) = client.whoami().await {
                        auth.set_active_session(session);
                    }
                    if is_spa_route(&redirect) {
                        navigate(&redirect, Default::default());
                    }
                }
                Err(err) => {
                    set_is_submitting.set(false);
                    match err.parse_kratos_flow() {
                        Some(updated) => {
                            let mut messages = updated.general_messages();
                            if let Some(e) = updated.field_error("password") {
                                messages.push(e);
                            }
                            set_error.set(Some(if messages.is_empty() {
                                t_or("common.auth.set_password_failed", "ตั้งรหัสผ่านไม่สำเร็จ")
                            } else {
                                messages.join(" ")
                            }));
                            set_flow.set(Some(updated));
                        }
                        None => set_error.set(Some(t_fmt("common.auth.set_password_failed_detail", "ตั้งรหัสผ่านไม่สำเร็จ: {error}", &[("error", &err.to_string())]))),
                    }
                }
            }
        });
    };
    let submit_on_enter = submit.clone();

    view! {
        <div class="min-h-screen w-full flex items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-950 dark:to-slate-900 p-4 relative">
            <div class="absolute top-4 right-4 z-10">
                <LanguageSwitcher />
            </div>
            <div class="w-full max-w-md">
                <Card class="border-border shadow-lg bg-card/95">
                    <CardHeader class="space-y-1 pb-4">
                        <CardTitle class="text-lg">{page_title}</CardTitle>
                        <CardDescription>{t_or("common.auth.set_password_description", "กำหนดรหัสผ่านสำหรับเข้าสู่ระบบ (อย่างน้อย 8 ตัวอักษร)")}</CardDescription>
                    </CardHeader>
                    <CardContent class="space-y-4">
                        {move || error.get().map(|err| view! {
                            <Alert variant=AlertVariant::Destructive class="py-2.5">
                                <AlertTitle class="text-xs font-semibold">{t_or("common.auth.action_failed", "ไม่สามารถดำเนินการได้")}</AlertTitle>
                                <AlertDescription class="text-xs mt-0.5">{err}</AlertDescription>
                            </Alert>
                        })}
                        {move || is_loading.get().then(|| view! {
                            <div class="flex items-center justify-center py-6 text-sm text-muted-foreground gap-2">
                                <Spinner class="h-4 w-4 text-primary" />
                                <span>{t_or("common.auth.verifying_link", "กำลังตรวจสอบลิงก์...")}</span>
                            </div>
                        })}
                        <form
                            class=move || if is_loading.get() || flow.get().is_none() { "hidden" } else { "space-y-4" }
                            on:submit=move |ev: web_sys::SubmitEvent| ev.prevent_default()
                        >
                            <div class="space-y-1.5">
                                <Label class="text-xs font-medium">{t_or("common.auth.new_password", "รหัสผ่านใหม่")}</Label>
                                <input
                                    type="password"
                                    name="password"
                                    autocomplete="new-password"
                                    disabled=move || is_submitting.get()
                                    class=INPUT_CLASS
                                    prop:value=move || password.get()
                                    on:input=move |ev| {
                                        set_password.set(event_target_value(&ev));
                                        set_field_error.set(None);
                                    }
                                />
                            </div>
                            <div class="space-y-1.5">
                                <Label class="text-xs font-medium">{t_or("common.auth.confirm_password", "ยืนยันรหัสผ่านใหม่")}</Label>
                                <input
                                    type="password"
                                    name="confirm"
                                    autocomplete="new-password"
                                    disabled=move || is_submitting.get()
                                    class=INPUT_CLASS
                                    prop:value=move || confirm.get()
                                    on:input=move |ev| {
                                        set_confirm.set(event_target_value(&ev));
                                        set_field_error.set(None);
                                    }
                                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            submit_on_enter();
                                        }
                                    }
                                />
                                {move || field_error.get().map(|msg| view! {
                                    <p class="text-[11px] text-destructive font-medium">{msg}</p>
                                })}
                            </div>
                            <Button
                                r#type="button"
                                variant=ButtonVariant::Default
                                class="w-full h-10 font-medium"
                                disabled=Signal::derive(move || is_submitting.get() || is_loading.get())
                                on_click=Callback::new(move |_: web_sys::MouseEvent| submit())
                            >
                                {move || if is_submitting.get() { t_or("common.auth.saving", "กำลังบันทึก...") } else { t_or("common.auth.save_password", "บันทึกรหัสผ่าน") }}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </div>
    }
}
