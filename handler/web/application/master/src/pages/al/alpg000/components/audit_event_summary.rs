//! Summary grid of one audit event (who, when, what, outcome).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle};

use crate::pages::al::alpg000::components::audit_log_table::{kind_label, kind_variant, status_variant};
use crate::pages::al::alpg000::components::audit_time_format::EventTime;
use crate::pages::al::alpg000::models::AuditLogDto;

fn text(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[component]
fn Field(label: String, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-0.5 min-w-0">
            <div class="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</div>
            <div class="text-sm break-all">{children()}</div>
        </div>
    }
}

fn mono_or_dash(value: Option<String>) -> impl IntoView {
    match value {
        Some(v) => view! { <span class="font-mono text-xs">{v}</span> }.into_any(),
        None => view! { <span class="text-muted-foreground">"-"</span> }.into_any(),
    }
}

#[component]
pub fn AuditEventSummary(event: AuditLogDto) -> impl IntoView {
    let time = EventTime::parse(&event.timestamp);
    let when = match &time {
        Some(t) => view! {
            <span class="font-mono text-xs">{t.local_text()}</span>
            <span class="block text-[11px] text-muted-foreground font-mono">{t.utc.clone()}</span>
        }
        .into_any(),
        None => view! { <span class="font-mono text-xs">{event.timestamp.clone()}</span> }.into_any(),
    };

    let request_line = {
        let method = text(&event.method);
        let path = text(&event.path);
        let query = text(&event.query);
        if method.is_none() && path.is_none() {
            None
        } else {
            let mut line = format!("{} {}", method.unwrap_or_default(), path.unwrap_or_default());
            if let Some(q) = query {
                line.push('?');
                line.push_str(q.trim_start_matches('?'));
            }
            Some(line.trim().to_string())
        }
    };
    let status = event.status.map(|s| {
        view! { <Badge variant=status_variant(s) class="text-xs font-mono">{s.to_string()}</Badge> }
    });
    let duration = event.duration_ms.map(|d| format!("{d} ms"));

    view! {
        <Card>
            <CardHeader class="pb-3">
                <CardTitle class="text-base">{t_or("alpg000.detail.summary", "สรุปเหตุการณ์")}</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="grid gap-x-6 gap-y-4 sm:grid-cols-2 lg:grid-cols-3">
                    <Field label=t_or("alpg000.field.event_id", "รหัสเหตุการณ์")>{mono_or_dash(text(&Some(event.event_id.clone())))}</Field>
                    <Field label=t_or("alpg000.field.time", "เวลา (ท้องถิ่น / UTC)")>{when}</Field>
                    <Field label=t_or("alpg000.field.kind", "ประเภท")>
                        <Badge variant=kind_variant(&event.kind) class="text-xs">{kind_label(&event.kind)}</Badge>
                    </Field>
                    <Field label=t_or("alpg000.field.user_id", "รหัสผู้ใช้")>{mono_or_dash(text(&event.user_id))}</Field>
                    <Field label=t_or("alpg000.field.username", "ชื่อผู้ใช้")>{mono_or_dash(text(&event.username))}</Field>
                    <Field label=t_or("alpg000.field.email", "อีเมล")>{mono_or_dash(text(&event.email))}</Field>
                    <Field label=t_or("alpg000.field.session_id", "รหัสเซสชัน")>{mono_or_dash(text(&event.session_id))}</Field>
                    <Field label=t_or("alpg000.field.ip", "IP")>{mono_or_dash(text(&event.ip))}</Field>
                    <Field label=t_or("alpg000.field.language", "ภาษา")>{mono_or_dash(text(&event.language))}</Field>
                    <Field label=t_or("alpg000.field.user_agent", "User agent")>{mono_or_dash(text(&event.user_agent))}</Field>
                    <Field label=t_or("alpg000.field.program", "โปรแกรม")>
                        {match text(&event.program_code) {
                            Some(p) => view! { <Badge variant=BadgeVariant::Outline class="text-xs font-mono">{p}</Badge> }.into_any(),
                            None => view! { <span class="text-muted-foreground">"-"</span> }.into_any(),
                        }}
                    </Field>
                    <Field label=t_or("alpg000.field.action", "การกระทำ")>{mono_or_dash(text(&event.action))}</Field>
                    <Field label=t_or("alpg000.field.request", "คำขอ (method + path + query)")>{mono_or_dash(request_line)}</Field>
                    <Field label=t_or("alpg000.field.status", "สถานะ")>
                        {match status {
                            Some(s) => s.into_any(),
                            None => view! { <span class="text-muted-foreground">"-"</span> }.into_any(),
                        }}
                    </Field>
                    <Field label=t_or("alpg000.field.duration", "ระยะเวลา")>{mono_or_dash(duration)}</Field>
                </div>
            </CardContent>
        </Card>
    }
}
