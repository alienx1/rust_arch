//! Active Sessions & Devices Management Card

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use web_package::ui::component::{
    Badge, BadgeVariant, Button, ButtonVariant, Card, CardContent, CardDescription, CardHeader,
    CardTitle,
};

use crate::pages::su::supg005::models::ActiveDeviceDto;
use crate::pages::su::supg005::providers::use_supg005;

#[component]
pub fn SessionDevicesCard(devices: Vec<ActiveDeviceDto>) -> impl IntoView {
    let ctx = use_supg005();
    let (revoked_msg, set_revoked_msg) = signal(false);

    let on_revoke_others = move |_| {
        ctx.revoke_other_sessions.run(());
        set_revoked_msg.set(true);
    };

    view! {
        <Card class="border-border/60 shadow-sm bg-card">
            <CardHeader class="pb-3 border-b border-border/40">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <CardTitle class="text-base font-semibold flex items-center gap-2">
                            <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                                <line x1="8" y1="21" x2="16" y2="21" />
                                <line x1="12" y1="17" x2="12" y2="21" />
                            </svg>
                            {t_or("supg005.devices.title", "อุปกรณ์และเซสชันของบัญชีนี้ (My Active Devices & Sessions)")}
                        </CardTitle>
                        <CardDescription>
                            {t_or("supg005.devices.description", "เซสชันที่เชื่อมต่อกับ Ory Kratos อยู่ในปัจจุบัน คุณสามารถเพิกถอนสิทธิ์อุปกรณ์อื่นได้ทันทีเพื่อความปลอดภัย")}
                        </CardDescription>
                    </div>

                    <Button
                        variant=ButtonVariant::Destructive
                        class="text-xs h-8 gap-1.5 self-start sm:self-auto"
                        on_click=Callback::new(on_revoke_others)
                    >
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                        </svg>
                        {t_or("supg005.devices.revoke_others", "เพิกถอนเซสชันอุปกรณ์อื่นทั้งหมด")}
                    </Button>
                </div>
            </CardHeader>
            <CardContent class="pt-4 space-y-3">
                {move || {
                    if revoked_msg.get() {
                        view! {
                            <div class="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-700 dark:text-emerald-400 text-xs flex items-center gap-2">
                                <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path d="M5 13l4 4L19 7" />
                                </svg>
                                <span>{t_or("supg005.devices.revoked_notice", "เพิกถอนเซสชันอุปกรณ์อื่นทั้งหมดเรียบร้อยแล้ว เซสชันใน Ory Kratos ถูกยกเลิกทันที")}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}

                <div class="divide-y divide-border/40">
                    {devices.into_iter().map(|dev| {
                        let is_curr = dev.is_current;
                        let dev_id = dev.id.clone();
                        let dev_uid = dev.user_id.clone();

                        view! {
                            <div class="py-3 flex items-center justify-between gap-4">
                                <div class="flex items-center gap-3">
                                    <div class=if is_curr {
                                        "w-8 h-8 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shrink-0"
                                    } else {
                                        "w-8 h-8 rounded-full bg-muted text-muted-foreground flex items-center justify-center shrink-0"
                                    }>
                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                                            <line x1="8" y1="21" x2="16" y2="21" />
                                            <line x1="12" y1="17" x2="12" y2="21" />
                                        </svg>
                                    </div>
                                    <div>
                                        <div class="flex items-center gap-2">
                                            <span class="font-medium text-xs text-foreground">{dev.device_name.clone()}</span>
                                            {if is_curr {
                                                view! {
                                                    <Badge variant=BadgeVariant::Success class="text-[10px] py-0 px-1.5 font-mono">
                                                        {t_or("supg005.devices.current", "อุปกรณ์ปัจจุบัน")}
                                                    </Badge>
                                                }.into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }}
                                        </div>
                                        <p class="text-[11px] text-muted-foreground mt-0.5">
                                            <span>{dev.ip_address.clone()}</span>
                                            <span>" • "</span>
                                            <span>{dev.location.clone()}</span>
                                        </p>
                                    </div>
                                </div>

                                <div class="flex items-center gap-3">
                                    <div class="text-right text-[11px]">
                                        <span class="text-muted-foreground block">{t_or("supg005.devices.last_active", "ใช้งานล่าสุด")}</span>
                                        <span class="font-medium text-foreground">{dev.last_active.clone()}</span>
                                    </div>

                                    {if !is_curr {
                                        view! {
                                            <Button
                                                variant=ButtonVariant::Outline
                                                class="text-xs h-7 px-2 text-destructive hover:bg-destructive/10 hover:border-destructive/30 shrink-0 gap-1"
                                                on_click=Callback::new(move |_| {
                                                    ctx.revoke_device.run((dev_uid.clone(), dev_id.clone()));
                                                })
                                            >
                                                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                    <line x1="18" y1="6" x2="6" y2="18" />
                                                    <line x1="6" y1="6" x2="18" y2="18" />
                                                </svg>
                                                <span>{t_or("supg005.devices.revoke", "เพิกถอน")}</span>
                                            </Button>
                                        }.into_any()
                                    } else {
                                        view! { <span /> }.into_any()
                                    }}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </CardContent>
        </Card>
    }
}
