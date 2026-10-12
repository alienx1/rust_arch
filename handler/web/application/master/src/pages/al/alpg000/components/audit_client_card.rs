//! Page / element / label / detail of a client-reported event (page view, click, action).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardHeader, CardTitle};

use crate::pages::al::alpg000::models::AuditLogDto;

fn row(label: String, value: &Option<String>) -> impl IntoView + use<> {
    let value = value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    view! {
        <div class="space-y-0.5 min-w-0">
            <div class="text-[11px] uppercase tracking-wide text-muted-foreground">{label}</div>
            <div class="text-sm font-mono break-all">{value.unwrap_or_else(|| "-".to_string())}</div>
        </div>
    }
}

#[component]
pub fn AuditClientCard(event: AuditLogDto) -> impl IntoView {
    view! {
        <Card>
            <CardHeader class="pb-3">
                <CardTitle class="text-base">{t_or("alpg000.detail.client", "ข้อมูลการใช้งานหน้าจอ")}</CardTitle>
            </CardHeader>
            <CardContent>
                <div class="grid gap-x-6 gap-y-4 sm:grid-cols-2">
                    {row(t_or("alpg000.field.page", "หน้า"), &event.page)}
                    {row(t_or("alpg000.field.element", "องค์ประกอบ"), &event.element)}
                    {row(t_or("alpg000.field.label", "ข้อความ"), &event.label)}
                    {row(t_or("alpg000.field.detail", "รายละเอียด"), &event.detail)}
                </div>
            </CardContent>
        </Card>
    }
}
