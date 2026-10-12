//! Ory Kratos Headless Self-Service Login Page component for Leptos SPA.
//!
//! Provides a responsive, enterprise-grade authentication interface compatible with
//! Ory Kratos browser login flows (`/self-service/login/browser`).

use super::context::use_auth;
use super::types::{KratosLoginFlow, LoginPayload};
use crate::ui::component::{
    Alert, AlertDescription, AlertTitle, AlertVariant, Button, ButtonVariant, Card, CardContent,
    CardDescription, CardFooter, CardHeader, CardTitle, Checkbox, Label, Spinner,
};
use crate::core::i18n::{LanguageSwitcher, t_fmt, t_or};
use leptos::prelude::*;

/// Browser helper to read query parameters safely.
pub(crate) fn get_query_param(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(search) = win.location().search() {
                let clean = search.trim_start_matches('?');
                for pair in clean.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        if k == name {
                            return Some(percent_decode_str(v));
                        }
                    }
                }
            }
        }
    }
    let _ = name;
    None
}

/// Returns `true` for same-origin SPA routes that the client router can handle
/// without a full page reload (e.g. `/`, `/demo`), excluding proxied API paths.
pub(crate) fn is_spa_route(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//") && !url.starts_with("/api/")
}

/// Browser helper to perform a full-page redirect (external / Kratos URLs).
fn redirect_to_url(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            let _ = win.location().set_href(url);
        }
    }
    let _ = url;
}

/// Simple percent decoder for query strings.
#[allow(dead_code)]
fn percent_decode_str(input: &str) -> String {
    let replaced = input.replace('+', " ");
    let mut result = String::with_capacity(replaced.len());
    let mut chars = replaced.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(b) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                    result.push(b as char);
                    continue;
                }
            }
        }
        result.push(c);
    }
    result
}

/// Ory Kratos Login Page Component.
///
/// Handles Kratos self-service login flow lifecycle:
/// 1. Reads existing `flow` query parameter or initializes a new flow via `/self-service/login/browser`.
/// 2. Dynamically renders CSRF token and credentials input fields.
/// 3. Submits credentials to the Kratos action endpoint.
/// 4. Handles 400 Bad Request validation errors and redirects on success.
#[component]
pub fn LoginPage(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] subtitle: Option<String>,
    #[prop(optional, into)] default_redirect: Option<String>,
) -> impl IntoView {
    let auth = use_auth();
    let client = auth.client();
    let navigate = leptos_router::hooks::use_navigate();
    let go_to = move |url: &str| {
        if is_spa_route(url) {
            navigate(
                url,
                leptos_router::NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        } else {
            redirect_to_url(url);
        }
    };
    let go_to_on_submit = go_to.clone();

    let page_title = title.unwrap_or_else(|| t_or("common.auth.login_title", "เข้าสู่ระบบ"));
    let page_subtitle =
        subtitle.unwrap_or_else(|| {
            t_or("common.auth.login_subtitle", "ระบบจัดการข้อมูลหลัก (Master Data Management Engine)")
        });
    let target_redirect = default_redirect.unwrap_or_else(|| "/".to_string());
    let redirect_on_mount = target_redirect.clone();
    let redirect_on_submit = target_redirect;

    // Reactive form state
    let (flow, set_flow) = signal::<Option<KratosLoginFlow>>(None);
    let (is_loading_flow, set_is_loading_flow) = signal::<bool>(true);
    let (is_submitting, set_is_submitting) = signal::<bool>(false);

    let (identifier, set_identifier) = signal::<String>(String::new());
    let (password, set_password) = signal::<String>(String::new());
    let (show_password, set_show_password) = signal::<bool>(false);
    let (remember_me, set_remember_me) = signal::<bool>(false);

    let (general_error, set_general_error) = signal::<Option<String>>(None);
    let (identifier_error, set_identifier_error) = signal::<Option<String>>(None);
    let (password_error, set_password_error) = signal::<Option<String>>(None);

    // 1. Initial Flow Setup
    Effect::new(move |_| {
        // If already authenticated, redirect immediately
        if auth.is_authenticated.get() {
            let redirect_target =
                get_query_param("return_to").unwrap_or_else(|| redirect_on_mount.clone());
            go_to(&redirect_target);
            return;
        }

        set_is_loading_flow.set(true);
        leptos::task::spawn_local(async move {
            let flow_param = get_query_param("flow");
            let result = if let Some(flow_id) = flow_param {
                client.get_login_flow(&flow_id).await
            } else {
                client.init_login_flow().await
            };

            match result {
                Ok(new_flow) => {
                    let messages = new_flow.general_messages();
                    if !messages.is_empty() {
                        set_general_error.set(Some(messages.join(" ")));
                    }
                    set_flow.set(Some(new_flow));
                }
                Err(e) => {
                    set_general_error.set(Some(t_fmt("common.auth.connect_failed", "ไม่สามารถเชื่อมต่อระบบยืนยันตัวตนได้: {error}", &[("error", &e.to_string())])));
                }
            }
            set_is_loading_flow.set(false);
        });
    });

    // 2. Submit Logic
    let do_submit = std::sync::Arc::new(move || {
        let ident = identifier.get().trim().to_string();
        let pass = password.get();

        // Basic client validation
        let mut has_client_error = false;
        if ident.is_empty() {
            set_identifier_error.set(Some(t_or("common.auth.identifier_required", "กรุณากรอกชื่อผู้ใช้หรืออีเมล")));
            has_client_error = true;
        } else {
            set_identifier_error.set(None);
        }

        if pass.is_empty() {
            set_password_error.set(Some(t_or("common.auth.password_required", "กรุณากรอกรหัสผ่าน")));
            has_client_error = true;
        } else {
            set_password_error.set(None);
        }

        if has_client_error {
            return;
        }

        let current_flow = match flow.get() {
            Some(f) => f,
            None => {
                set_general_error.set(Some(t_or("common.auth.flow_not_ready", "ระบบเข้าสู่ระบบยังไม่พร้อม กรุณาลองใหม่อีกครั้ง")));
                return;
            }
        };

        let action_url = current_flow.ui.action.clone();
        let csrf_token = current_flow.csrf_token();

        let mut payload = LoginPayload::new(ident, pass);
        if let Some(token) = csrf_token {
            payload = payload.with_csrf(token);
        }

        set_is_submitting.set(true);
        set_general_error.set(None);

        let redirect_url =
            get_query_param("return_to").unwrap_or_else(|| redirect_on_submit.clone());

        let go_to = go_to_on_submit.clone();
        leptos::task::spawn_local(async move {
            match client.submit_login(&action_url, &payload).await {
                Ok(session) => {
                    auth.set_active_session(session);
                    go_to(&redirect_url);
                }
                Err(err) => {
                    set_is_submitting.set(false);

                    // Check if Kratos returned an updated flow payload with validation errors
                    if let Some(updated_flow) = err.parse_kratos_flow() {
                        let general_msgs = updated_flow.general_messages();
                        if !general_msgs.is_empty() {
                            set_general_error.set(Some(general_msgs.join(" ")));
                        } else {
                            set_general_error.set(Some(t_or("common.auth.invalid_credentials", "ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง")));
                        }

                        if let Some(id_err) = updated_flow.field_error("identifier") {
                            set_identifier_error.set(Some(id_err));
                        }
                        if let Some(pw_err) = updated_flow.field_error("password") {
                            set_password_error.set(Some(pw_err));
                        }

                        // Refresh flow so new CSRF token is retained
                        set_flow.set(Some(updated_flow));
                    } else {
                        set_general_error.set(Some(t_fmt("common.auth.login_error", "เกิดข้อผิดพลาดในการเข้าสู่ระบบ: {error}", &[("error", &err.to_string())])));
                    }
                }
            }
        });
    });

    let submit_form = do_submit.clone();
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        submit_form();
    };

    let submit_btn = do_submit.clone();
    let on_button_click = Callback::new(move |_: web_sys::MouseEvent| {
        submit_btn();
    });

    let submit_key1 = do_submit.clone();
    let submit_key2 = do_submit.clone();

    let is_btn_disabled = Signal::derive(move || is_submitting.get() || is_loading_flow.get());

    view! {
        <div class="min-h-screen w-full flex items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-950 dark:to-slate-900 p-4 relative">
            <div class="absolute top-4 right-4 z-10">
                <LanguageSwitcher />
            </div>
            <div class="w-full max-w-md">
                // Brand Header with Shield Icon
                <div class="flex flex-col items-center text-center mb-6">
                    <div class="h-12 w-12 rounded-xl bg-primary/10 text-primary flex items-center justify-center mb-3 shadow-inner ring-1 ring-primary/20">
                        <svg class="h-7 w-7 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                            <path d="m9 12 2 2 4-4" />
                        </svg>
                    </div>
                    <h1 class="text-2xl font-bold tracking-tight text-foreground">{page_title}</h1>
                    <p class="text-sm text-muted-foreground mt-1 max-w-xs">{page_subtitle}</p>
                </div>

                // Main Login Card
                <Card class="border-border shadow-lg bg-card/95 backdrop-blur-sm">
                    <CardHeader class="space-y-1 pb-4">
                        <CardTitle class="text-lg">{t_or("common.auth.card_title", "ลงชื่อเข้าใช้งาน")}</CardTitle>
                        <CardDescription>
                            {t_or("common.auth.card_description", "กรอกข้อมูลประจำตัวของคุณเพื่อเข้าสู่ระบบ")}
                        </CardDescription>
                    </CardHeader>

                    <CardContent class="space-y-4">
                        // General Error Alert
                        {move || general_error.get().map(|err| view! {
                            <Alert variant=AlertVariant::Destructive class="py-2.5">
                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <circle cx="12" cy="12" r="10" />
                                    <line x1="12" y1="8" x2="12" y2="12" />
                                    <line x1="12" y1="16" x2="12.01" y2="16" />
                                </svg>
                                <AlertTitle class="text-xs font-semibold">{t_or("common.auth.login_blocked", "ไม่สามารถเข้าสู่ระบบได้")}</AlertTitle>
                                <AlertDescription class="text-xs mt-0.5">
                                    {err}
                                </AlertDescription>
                            </Alert>
                        })}

                        // Flow Loading Indicator
                        {move || is_loading_flow.get().then(|| view! {
                            <div class="flex items-center justify-center py-6 text-sm text-muted-foreground gap-2">
                                <Spinner class="h-4 w-4 text-primary" />
                                <span>{t_or("common.auth.preparing", "กำลังเตรียมความพร้อมของระบบความปลอดภัย...")}</span>
                            </div>
                        })}

                        // Login Form
                        <form
                            class=move || if is_loading_flow.get() { "hidden" } else { "space-y-4" }
                            on:submit=on_submit
                        >
                            // Identifier (Username / Email) Field
                            <div class="space-y-1.5">
                                <Label class="text-xs font-medium">{t_or("common.auth.identifier", "ชื่อผู้ใช้ หรือ อีเมล")}</Label>
                                <div class="relative flex items-center">
                                    <span class="absolute left-3 text-muted-foreground pointer-events-none flex items-center">
                                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                            <rect width="20" height="16" x="2" y="4" rx="2" />
                                            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
                                        </svg>
                                    </span>
                                    <input
                                        type="text"
                                        name="identifier"
                                        autocomplete="username"
                                        placeholder="admin@arch.local"
                                        disabled=move || is_submitting.get()
                                        class="flex h-9 w-full rounded-md border border-input bg-transparent pl-9 pr-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                                        prop:value=move || identifier.get()
                                        on:input=move |ev| {
                                            set_identifier.set(event_target_value(&ev));
                                            set_identifier_error.set(None);
                                        }
                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                            if ev.key() == "Enter" {
                                                submit_key1();
                                            }
                                        }
                                    />
                                </div>
                                {move || identifier_error.get().map(|msg| view! {
                                    <p class="text-[11px] text-destructive font-medium">{msg}</p>
                                })}
                            </div>

                            // Password Field
                            <div class="space-y-1.5">
                                <div class="flex items-center justify-between">
                                    <Label class="text-xs font-medium">{t_or("common.auth.password", "รหัสผ่าน")}</Label>
                                    <a
                                        href="/recovery"
                                        class="text-xs text-primary hover:underline font-medium"
                                        tabindex="-1"
                                    >
                                        {t_or("common.auth.forgot_password", "ลืมรหัสผ่าน?")}
                                    </a>
                                </div>
                                <div class="relative flex items-center">
                                    <span class="absolute left-3 text-muted-foreground pointer-events-none flex items-center">
                                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                            <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
                                            <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                                        </svg>
                                    </span>
                                    <input
                                        type=move || if show_password.get() { "text" } else { "password" }
                                        name="password"
                                        autocomplete="current-password"
                                        placeholder="••••••••"
                                        disabled=move || is_submitting.get()
                                        class="flex h-9 w-full rounded-md border border-input bg-transparent pl-9 pr-9 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                                        prop:value=move || password.get()
                                        on:input=move |ev| {
                                            set_password.set(event_target_value(&ev));
                                            set_password_error.set(None);
                                        }
                                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                                            if ev.key() == "Enter" {
                                                submit_key2();
                                            }
                                        }
                                    />
                                    <button
                                        type="button"
                                        class="absolute right-2.5 text-muted-foreground hover:text-foreground cursor-pointer transition-colors p-0.5 rounded"
                                        tabindex="-1"
                                        on:click=move |_| set_show_password.update(|s| *s = !*s)
                                    >
                                        {move || if show_password.get() {
                                            view! {
                                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                    <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
                                                    <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
                                                    <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
                                                    <line x1="2" y1="2" x2="22" y2="22" />
                                                </svg>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                    <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                                                    <circle cx="12" cy="12" r="3" />
                                                </svg>
                                            }.into_any()
                                        }}
                                    </button>
                                </div>
                                {move || password_error.get().map(|msg| view! {
                                    <p class="text-[11px] text-destructive font-medium">{msg}</p>
                                })}
                            </div>

                            // Remember Me Checkbox
                            <div class="flex items-center justify-between pt-1">
                                <Checkbox
                                    checked=Signal::derive(move || remember_me.get())
                                    on_change=Callback::new(move |c| set_remember_me.set(c))
                                    label=t_or("common.auth.remember_me", "จดจำการเข้าสู่ระบบ")
                                />
                            </div>

                            // Submit Button
                            <div class="pt-2">
                                <Button
                                    r#type="submit"
                                    variant=ButtonVariant::Default
                                    class="w-full h-10 font-medium"
                                    disabled=is_btn_disabled
                                    on_click=on_button_click
                                >
                                    {move || if is_submitting.get() {
                                        view! {
                                            <span class="inline-flex items-center gap-2">
                                                <Spinner class="h-4 w-4 text-primary-foreground" />
                                                <span>{t_or("common.auth.verifying", "กำลังตรวจสอบสิทธิ์...")}</span>
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="inline-flex items-center gap-2">
                                                <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                    <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                                                    <polyline points="10 17 15 12 10 7" />
                                                    <line x1="15" y1="12" x2="3" y2="12" />
                                                </svg>
                                                <span>{t_or("common.auth.login_title", "เข้าสู่ระบบ")}</span>
                                            </span>
                                        }.into_any()
                                    }}
                                </Button>
                            </div>
                        </form>
                    </CardContent>

                    <CardFooter class="flex flex-col border-t border-border pt-4 pb-4 text-center text-xs text-muted-foreground">
                        <div class="flex items-center justify-center gap-1.5 text-slate-500 dark:text-slate-400">
                            <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
                                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                            </svg>
                            <span>{t_or("common.auth.secured_by", "เข้ารหัสความปลอดภัยด้วย Ory Kratos & Argon2id")}</span>
                        </div>
                    </CardFooter>
                </Card>

                // Footer Info
                <div class="text-center mt-6 text-xs text-muted-foreground space-y-1">
                    <p>{t_or("common.auth.copyright", "© 2026 Enterprise Platform. สงวนลิขสิทธิ์ทั้งหมด")}</p>
                </div>
            </div>
        </div>
    }
}
