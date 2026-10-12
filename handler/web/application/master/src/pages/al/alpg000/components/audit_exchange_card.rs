//! "Request" / "Response" card: headers (already masked by the server) and the JSON body.

use leptos::prelude::*;
use serde_json::Value;
use web_package::core::i18n::t_or;
use web_package::ui::component::{Card, CardContent, CardHeader, CardTitle};

use crate::pages::al::alpg000::components::json_block::{HeaderTable, JsonBlock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExchangeSide {
    Request,
    Response,
}

#[component]
pub fn AuditExchangeCard(
    side: ExchangeSide,
    headers: Option<Value>,
    body: Option<Value>,
) -> impl IntoView {
    let title = match side {
        ExchangeSide::Request => t_or("alpg000.detail.request", "คำขอ (Request)"),
        ExchangeSide::Response => t_or("alpg000.detail.response", "คำตอบ (Response)"),
    };
    view! {
        <Card>
            <CardHeader class="pb-3">
                <CardTitle class="text-base">{title}</CardTitle>
            </CardHeader>
            <CardContent class="space-y-4">
                <div class="space-y-2">
                    <div class="text-xs font-medium text-muted-foreground">{t_or("alpg000.detail.headers", "ส่วนหัว (Headers)")}</div>
                    <HeaderTable headers=headers />
                </div>
                <div class="space-y-2">
                    <div class="text-xs font-medium text-muted-foreground">{t_or("alpg000.detail.body", "เนื้อหา (Body)")}</div>
                    <JsonBlock body=body />
                </div>
            </CardContent>
        </Card>
    }
}
