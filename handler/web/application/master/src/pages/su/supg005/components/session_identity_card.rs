//! Identity & User Profile Card Component

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{
    Avatar, Badge, BadgeVariant, Card, CardContent, CardHeader, CardTitle,
};

use crate::pages::su::supg005::models::UserIdentityDto;

#[component]
pub fn SessionIdentityCard(identity: UserIdentityDto) -> impl IntoView {
    let full_name = identity.full_name();
    let initials = identity.initials();
    let user_id = identity.user_id.clone();
    let (copied, set_copied) = signal(false);

    let on_copy_id = move |_| {
        set_copied.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let nav = window.navigator();
                let clipboard = nav.clipboard();
                let _ = clipboard.write_text(&user_id);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = &user_id;
    };

    view! {
        <Card class="border-border/60 shadow-sm bg-card hover:border-primary/30 transition-all">
            <CardHeader class="pb-3 border-b border-border/40">
                <div class="flex items-center justify-between">
                    <CardTitle class="text-base font-semibold flex items-center gap-2">
                        <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
                            <circle cx="12" cy="7" r="4" />
                        </svg>
                        {t_or("supg005.identity.title", "ข้อมูลตัวตนผู้ใช้งาน (User Identity)")}
                    </CardTitle>
                    <Badge variant=BadgeVariant::Success>
                        {identity.account_status.clone()}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent class="pt-4 space-y-4">
                <div class="flex items-center gap-3.5">
                    <Avatar
                        fallback=initials
                        class="h-12 w-12 border-2 border-primary/20 text-primary font-bold text-sm"
                    />
                    <div class="min-w-0 flex-1">
                        <h4 class="font-semibold text-foreground text-sm truncate">{full_name}</h4>
                        <p class="text-xs text-muted-foreground flex items-center gap-1.5 mt-0.5">
                            <span class="font-mono text-foreground/80">@{identity.username.clone()}</span>
                            <span>"•"</span>
                            <span>{identity.email.clone()}</span>
                        </p>
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-2 text-xs bg-muted/30 p-2.5 rounded-lg border border-border/40">
                    <div>
                        <span class="text-muted-foreground block text-[11px]">{t_or("supg005.identity.employee_code", "รหัสพนักงาน")}</span>
                        <span class="font-mono font-medium text-foreground">{identity.employee_code.clone()}</span>
                    </div>
                    <div>
                        <span class="text-muted-foreground block text-[11px]">{t_or("supg005.identity.account_status", "สถานะบัญชี")}</span>
                        <span class="font-medium text-emerald-600 dark:text-emerald-400">{t_or("supg005.identity.status_normal", "ปกติ (Active)")}</span>
                    </div>
                </div>

                <div class="pt-1">
                    <div class="flex items-center justify-between text-[11px] text-muted-foreground mb-1">
                        <span>"Identity UUID (Kratos / PostgreSQL)"</span>
                        <button
                            type="button"
                            class="text-primary hover:underline flex items-center gap-1 cursor-pointer font-mono"
                            on:click=on_copy_id
                        >
                            {move || if copied.get() { t_or("supg004.recovery.copied", "คัดลอกแล้ว ✓") } else { t_or("supg005.identity.copy", "คัดลอก") }}
                        </button>
                    </div>
                    <div class="font-mono text-[11px] bg-muted/60 p-2 rounded border border-border/50 break-all select-all text-muted-foreground">
                        {identity.user_id.clone()}
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}
