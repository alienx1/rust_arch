//! Session Statistics & KPI Cards Component

use web_package::core::i18n::{t_or, t_fmt};
use leptos::prelude::*;
use web_package::ui::component::{Card, CardContent};

use crate::pages::su::supg005::models::Supg005SessionSummary;

#[component]
pub fn SessionStatsKpis(summary: Supg005SessionSummary) -> impl IntoView {
    let total_users = summary.total_users_count();
    let online_users = summary.total_online_users_count();
    let active_devices = summary.total_active_devices_count();
    let is_live = summary.is_live_session;

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            // Card 1: ผู้ใช้งานทั้งหมด
            <Card class="border-border/60 bg-card shadow-sm hover:border-primary/30 transition-all">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <span class="text-xs text-muted-foreground block">{t_or("supg005.kpi.total_users", "ผู้ใช้งานทั้งหมดในระบบ")}</span>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold text-foreground font-mono">{total_users}</span>
                            <span class="text-xs text-muted-foreground">{t_or("supg005.kpi.accounts", "บัญชี")}</span>
                        </div>
                    </div>
                    <div class="w-10 h-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
                            <circle cx="9" cy="7" r="4" />
                            <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 2: ผู้ใช้ออนไลน์
            <Card class="border-border/60 bg-card shadow-sm hover:border-emerald-500/30 transition-all">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <span class="text-xs text-muted-foreground block">{t_or("supg005.kpi.online_users", "ผู้ใช้ออนไลน์ขณะนี้")}</span>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold text-emerald-600 dark:text-emerald-400 font-mono">{online_users}</span>
                            <span class="text-xs text-emerald-600/80 dark:text-emerald-400/80 font-medium">
                                {t_fmt("supg005.kpi.out_of", "จาก {total} คน", &[("total", &total_users.to_string())])}
                            </span>
                        </div>
                    </div>
                    <div class="w-10 h-10 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <circle cx="12" cy="12" r="10" />
                            <circle cx="12" cy="12" r="3" fill="currentColor" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 3: อุปกรณ์ที่เชื่อมต่อทั้งหมด
            <Card class="border-border/60 bg-card shadow-sm hover:border-blue-500/30 transition-all">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <span class="text-xs text-muted-foreground block">{t_or("supg005.kpi.sessions_devices", "เซสชัน / อุปกรณ์ออนไลน์")}</span>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-2xl font-bold text-blue-600 dark:text-blue-400 font-mono">{active_devices}</span>
                            <span class="text-xs text-muted-foreground">{t_or("supg005.kpi.sessions", "เซสชัน")}</span>
                        </div>
                    </div>
                    <div class="w-10 h-10 rounded-lg bg-blue-500/10 text-blue-600 dark:text-blue-400 flex items-center justify-center">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                            <line x1="8" y1="21" x2="16" y2="21" />
                            <line x1="12" y1="17" x2="12" y2="21" />
                        </svg>
                    </div>
                </CardContent>
            </Card>

            // Card 4: สถานะ Kratos Identity Store
            <Card class="border-border/60 bg-card shadow-sm hover:border-purple-500/30 transition-all">
                <CardContent class="p-4 flex items-center justify-between">
                    <div>
                        <span class="text-xs text-muted-foreground block">{t_or("supg005.kpi.iam_security", "ระบบความปลอดภัย IAM")}</span>
                        <div class="flex items-baseline gap-2 mt-1">
                            <span class="text-base font-bold text-foreground">
                                {if is_live { "Ory Kratos 1.3" } else { "Dev Simulation" }}
                            </span>
                        </div>
                        <span class="text-[11px] text-muted-foreground">"Valkey Sub-ms Cached"</span>
                    </div>
                    <div class="w-10 h-10 rounded-lg bg-purple-500/10 text-purple-600 dark:text-purple-400 flex items-center justify-center">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
                        </svg>
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}
