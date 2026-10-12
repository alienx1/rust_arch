//! ALPG000 Log Screen (`/al/alpg000/log/:id`): one audit event in full.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};
use web_package::ui::component::{Button, ButtonSize, ButtonVariant, Spinner};

use crate::pages::al::alpg000::components::{
    AuditClientCard, AuditDataChangeCard, AuditEventSummary, AuditExchangeCard, ExchangeSide,
};
use crate::pages::al::alpg000::models::{AuditLogDto, KIND_API, KIND_DATA_CHANGE};
use crate::pages::al::alpg000::providers::provide_alpg000_detail_context;

#[component]
pub fn Alpg000LogScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id").unwrap_or_default();
    let ctx = provide_alpg000_detail_context(id.clone());

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("alpg000.detail.title", "รายละเอียดเหตุการณ์ (Audit Event)")}</PageTitle>
                    <PageDescription>
                        <span class="font-mono">{id}</span>
                    </PageDescription>
                </div>
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    class="h-9 text-xs"
                    on_click=Callback::new(move |_| ctx.back.run(()))
                >
                    {t_or("alpg000.action.back", "← กลับ")}
                </Button>
            </PageHeader>
            <PageContent>
                {move || {
                    if ctx.is_loading.get() {
                        return view! {
                            <div class="flex items-center gap-2 text-xs text-muted-foreground py-8 justify-center">
                                <Spinner class="h-4 w-4" />
                                {t_or("alpg000.state.loading_detail", "กำลังโหลดรายละเอียด...")}
                            </div>
                        }.into_any();
                    }
                    if ctx.not_found.get() {
                        return view! {
                            <div class="p-3.5 rounded-lg bg-warning/10 border border-warning/30 text-xs">
                                {t_or("alpg000.error.not_found", "ไม่พบเหตุการณ์ที่ต้องการ")}
                            </div>
                        }.into_any();
                    }
                    if let Some(err) = ctx.error.get() {
                        return view! {
                            <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs font-medium">
                                {err}
                            </div>
                        }.into_any();
                    }
                    match ctx.event.get() {
                        Some(event) => event_view(event).into_any(),
                        None => ().into_any(),
                    }
                }}
            </PageContent>
        </PageContainer>
    }
}

/// Summary plus the exchange cards (api events) or the client card (page view / click / action)
fn event_view(event: AuditLogDto) -> impl IntoView {
    let has_exchange = event.kind == KIND_API
        || event.request_headers.is_some()
        || event.request_body.is_some()
        || event.response_headers.is_some()
        || event.response_body.is_some();
    let is_change = event.kind == KIND_DATA_CHANGE;
    let has_client = event.kind != KIND_API
        && !is_change
        && (event.page.is_some() || event.element.is_some() || event.label.is_some() || event.detail.is_some());

    view! {
        <div class="space-y-4">
            <AuditEventSummary event=event.clone() />
            {is_change.then(|| view! { <AuditDataChangeCard event=event.clone() /> })}
            {has_client.then(|| view! { <AuditClientCard event=event.clone() /> })}
            {has_exchange.then(|| view! {
                <div class="grid gap-4 xl:grid-cols-2">
                    <AuditExchangeCard
                        side=ExchangeSide::Request
                        headers=event.request_headers.clone()
                        body=event.request_body.clone()
                    />
                    <AuditExchangeCard
                        side=ExchangeSide::Response
                        headers=event.response_headers.clone()
                        body=event.response_body.clone()
                    />
                </div>
            })}
        </div>
    }
}
