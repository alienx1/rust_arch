//! Session Security & Ory Kratos Authenticator Card

use web_package::core::i18n::{t_or, t_fmt};
use leptos::prelude::*;
use web_package::ui::component::{
    Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle,
};

use crate::pages::su::supg005::models::SessionSecurityDto;

#[component]
pub fn SessionSecurityCard(security: SessionSecurityDto) -> impl IntoView {
    view! {
        <Card class="border-border/60 shadow-sm bg-card hover:border-primary/30 transition-all">
            <CardHeader class="pb-3 border-b border-border/40">
                <div class="flex items-center justify-between">
                    <CardTitle class="text-base font-semibold flex items-center gap-2">
                        <svg class="w-4 h-4 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                        </svg>
                        {t_or("supg005.security.title", "ความปลอดภัยเซสชัน (Session Security)")}
                    </CardTitle>
                    <Badge variant=BadgeVariant::Success>
                        "Session Active"
                    </Badge>
                </div>
            </CardHeader>
            <CardContent class="pt-4 space-y-3.5 text-xs">
                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.security.aal", "ระดับความปลอดภัย (AAL)")}</span>
                    <Badge variant=BadgeVariant::Secondary class="font-mono text-[11px]">
                        {security.aal_level.clone()}
                    </Badge>
                </div>

                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.security.auth_method", "วิธีพิสูจน์ตัวตน (Auth Method)")}</span>
                    <span class="font-medium text-foreground">{security.auth_method.clone()}</span>
                </div>

                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.security.authenticated_at", "เข้าสู่ระบบเมื่อ (Authenticated At)")}</span>
                    <span class="font-mono text-muted-foreground text-[11px]">{security.authenticated_at.clone()}</span>
                </div>

                <div class="flex items-center justify-between py-1 border-b border-border/30">
                    <span class="text-muted-foreground">{t_or("supg005.security.expires_at", "หมดอายุ (Expires At)")}</span>
                    <div class="text-right">
                        <span class="font-mono text-muted-foreground text-[11px] block">{security.expires_at.clone()}</span>
                        <span class="text-[10px] text-emerald-600 dark:text-emerald-400 font-medium">
                            {t_fmt("supg005.security.remaining", "เหลือเวลา: {time}", &[("time", &security.remaining_time.to_string())])}
                        </span>
                    </div>
                </div>

                <div class="pt-1">
                    <span class="text-muted-foreground block text-[11px] mb-1">"Ory Kratos Session ID"</span>
                    <div class="font-mono text-[11px] bg-muted/60 p-2 rounded border border-border/50 break-all select-all text-muted-foreground">
                        {security.session_id.clone()}
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
