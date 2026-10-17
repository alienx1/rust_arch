//! All Users & Sessions Management Table Component (shared `DataTable` shell)
//!
//! Provides directory of enterprise users with Server-Side SQL LIMIT & OFFSET:
//! - Queries backend API `/api/v1/su/supg005/users?page=...&limit=5`
//! - Does NOT dump entire dataset into memory
//! - Displays active device sessions per user
//! - Revokes individual device sessions (Revoke Device)
//! - Revokes all sessions for a user (Revoke User)

use leptos::prelude::*;
use web_package::core::i18n::{t_or, t_fmt};
use web_package::ui::component::{ConfirmButton, 
    Avatar, Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataColumn, DataTable,
    FilterChips, TableCell, TableRow,
};

use crate::access::{Action, PermissionGate};
use crate::pages::su::supg005::models::UserSessionOverviewDto;
use crate::pages::su::supg005::providers::use_supg005;

#[component]
pub fn SessionUsersTable(
    #[prop(optional, into)]
    #[allow(unused_variables)]
    users: Option<Vec<UserSessionOverviewDto>>,
) -> impl IntoView {
    let ctx = use_supg005();

    let on_search = move |val: String| {
        ctx.set_current_page.set(1);
        ctx.set_user_search_query.set(val);
    };

    let on_account_status_change = move |s: String| {
        ctx.set_current_page.set(1);
        ctx.set_account_status_filter.set(s);
    };

    let paginated_data = Signal::derive(move || ctx.paginated_users.get());
    let is_loading = Signal::derive(move || ctx.is_users_loading.get());

    let current_page = Signal::derive(move || ctx.current_page.get());
    let page_size = Signal::derive(move || ctx.page_size.get());

    let total_items = Signal::derive(move || {
        paginated_data.get().map(|d| d.total).unwrap_or(0)
    });

    let users_to_render = Signal::derive(move || {
        paginated_data.get().map(|d| d.items).unwrap_or_default()
    });

    let columns = vec![
        DataColumn::new(t_or("common.column.num", "#")).class("w-12 text-center"),
        DataColumn::new(t_or("supg005.column.user_account", "ผู้ใช้งาน / บัญชี")).class("min-w-[200px]"),
        DataColumn::new(t_or("supg005.column.dept_company", "แผนก & บริษัท")).class("min-w-[170px]"),
        DataColumn::new(t_or("supg005.column.role", "บทบาท (Role)")).class("min-w-[140px]"),
        DataColumn::new(t_or("supg005.column.status_devices", "สถานะ & เซสชันอุปกรณ์ (Active Devices)")).class("min-w-[280px]"),
        DataColumn::new(t_or("supg005.column.actions", "การจัดการ")).class("w-32 text-right"),
    ];

    view! {
        <div class="space-y-4">
            <DataTable
                title=t_or("supg005.table.title", "ตารางรายชื่อผู้ใช้งานและเซสชัน (Server-Side SQL LIMIT 5)")
                description=t_or("supg005.table.description", "ดึงข้อมูลจากฐานข้อมูลแบบแบ่งหน้า (SQL LIMIT 5, OFFSET) ช่วยให้โหลดรวดเร็วแม้มีผู้ใช้งานจำนวนมาก และสั่งเพิกถอนเซสชัน (Revoke) ได้ทันที")
                icon_path="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"
                columns=columns
                loading=is_loading
                empty=Signal::derive(move || users_to_render.get().is_empty())
                loading_text=t_or("supg005.table.loading", "กำลังดึงข้อมูลจาก SQL Database (LIMIT 5)...")
                empty_title=t_or("supg005.table.empty_title", "ไม่พบผู้ใช้งาน")
                empty_text=t_or("supg005.table.empty_text", "ไม่พบผู้ใช้งานที่ตรงกับเงื่อนไขการค้นหา")
                search_value=ctx.user_search_query
                search_placeholder=t_or("supg005.search.placeholder", "ค้นหาชื่อ, username...")
                on_search=Callback::new(on_search)
                on_refresh=ctx.refresh_users
                filters=move || view! {
                    <FilterChips
                        label=t_or("supg005.filter.account_status", "สถานะบัญชี:")
                        options=vec![
                            ("ALL".to_string(), t_or("supg005.filter.account_all", "ทั้งหมด")),
                            ("ACTIVE".to_string(), t_or("supg005.filter.account_active", "ใช้งานได้")),
                            ("SUSPENDED".to_string(), t_or("supg005.filter.account_suspended", "ถูกระงับ")),
                        ]
                        value=Signal::derive(move || ctx.account_status_filter.get())
                        on_change=Callback::new(on_account_status_change)
                    />
                }
                notice=move || view! {
                    {move || ctx.action_notification.get().map(|msg| view! {
                        <div class="p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-lg text-emerald-700 dark:text-emerald-400 text-xs flex items-center justify-between shadow-sm animate-in fade-in duration-200">
                            <div class="flex items-center gap-2">
                                <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                    <path d="M5 13l4 4L19 7" />
                                </svg>
                                <span class="font-medium">{msg}</span>
                            </div>
                            <button
                                type="button"
                                class="text-xs hover:underline cursor-pointer opacity-70 hover:opacity-100"
                                on:click=move |_| ctx.set_action_notification.set(None)
                            >
                                {t_or("supg005.table.close", "✕ ปิด")}
                            </button>
                        </div>
                    })}
                }
                page=current_page
                limit=page_size
                total=total_items
                on_page=Callback::new(move |p| ctx.set_current_page.set(p))
                on_limit=ctx.set_limit
            >
                {move || {
                    let curr_p = current_page.get();
                    let limit = page_size.get();
                    let users = users_to_render.get();
                    users.into_iter().enumerate().map(|(idx, u)| {
                                                let row_idx = (curr_p - 1) * limit + (idx as u64) + 1;
                                                let uid_for_revoke = u.user_id.clone();
                                                let is_suspended = !u.is_active_account;
                                                let is_self = web_package::core::auth::use_auth().user.get_untracked().and_then(|me| me.email()).as_deref() == Some(u.email.as_str());
                                                let uid_for_dev = u.user_id.clone();
                                                let initials = u.initials();
                                                let is_online = u.is_online();
                                                let active_count = u.active_sessions_count();
                                                let devices = u.devices.clone();

                                                view! {
                                                    <TableRow class="hover:bg-muted/30 transition-colors">
                                                        // 1. Index
                                                        <TableCell class="text-center font-mono text-xs text-muted-foreground">
                                                            {row_idx}
                                                        </TableCell>

                                                        // 2. User info
                                                        <TableCell>
                                                            <div class="flex items-center gap-3">
                                                                <Avatar
                                                                    fallback=initials
                                                                    class="h-9 w-9 border border-primary/20 text-primary font-bold text-xs shrink-0"
                                                                />
                                                                <div class="min-w-0">
                                                                    <div class="flex items-center gap-1.5 flex-wrap">
                                                                        <span class="font-semibold text-foreground text-xs">{u.full_name.clone()}</span>
                                                                        <span class="text-[11px] font-mono bg-muted px-1.5 py-0.5 rounded text-foreground/80">
                                                                            {u.employee_code.clone()}
                                                                        </span>
                                                                    </div>
                                                                    <div class="text-[11px] text-muted-foreground flex items-center gap-1 mt-0.5">
                                                                        <span class="font-mono">@{u.username.clone()}</span>
                                                                        <span>"•"</span>
                                                                        <span class="truncate">{u.email.clone()}</span>
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </TableCell>

                                                        // 3. Department & Company
                                                        <TableCell>
                                                            <div class="text-xs space-y-0.5">
                                                                <div class="font-medium text-foreground">{u.department_name.clone()}</div>
                                                                <div class="text-[11px] text-muted-foreground">{u.company_name.clone()}</div>
                                                                <div>
                                                                    <Badge variant=BadgeVariant::Outline class="text-[10px] px-1.5 py-0">
                                                                        {format!("Scope: {}", u.scope_level)}
                                                                    </Badge>
                                                                </div>
                                                            </div>
                                                        </TableCell>

                                                        // 4. Role
                                                        <TableCell>
                                                            <div class="space-y-1">
                                                                <Badge variant=BadgeVariant::Secondary class="text-xs font-mono">
                                                                    {u.role_code.clone()}
                                                                </Badge>
                                                                <div class="text-[11px] text-muted-foreground">{u.role_name.clone()}</div>
                                                            </div>
                                                        </TableCell>

                                                        // 5. Status & Devices
                                                        <TableCell>
                                                            <div class="space-y-2">
                                                                <div class="flex items-center gap-2">
                                                                    {if is_suspended {
                                                                        view! {
                                                                            <Badge variant=BadgeVariant::Destructive class="text-xs">
                                                                                {t_or("supg005.table.suspended", "ถูกระงับบัญชี")}
                                                                            </Badge>
                                                                        }.into_any()
                                                                    } else if is_online {
                                                                        view! {
                                                                            <Badge variant=BadgeVariant::Success class="text-xs flex items-center gap-1.5">
                                                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                                                                                {t_fmt("supg005.table.online_sessions", "ออนไลน์ ({count} เซสชัน)", &[("count", &active_count.to_string())])}
                                                                            </Badge>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <Badge variant=BadgeVariant::Outline class="text-xs text-muted-foreground">
                                                                                {t_or("supg005.filter.offline", "ออฟไลน์ (Offline)")}
                                                                            </Badge>
                                                                        }.into_any()
                                                                    }}
                                                                </div>

                                                                {if is_online && !devices.is_empty() {
                                                                    let dev_uid_capture = uid_for_dev.clone();
                                                                    view! {
                                                                        <div class="space-y-2 max-w-2xl">
                                                                            {devices.into_iter().map(|dev| {
                                                                                let d_uid = dev_uid_capture.clone();
                                                                                let dev_id = dev.id.clone();
                                                                                let is_curr = dev.is_current;

                                                                                view! {
                                                                                    <div class="flex items-center justify-between p-2 rounded-md bg-muted/40 border border-border/50 text-xs gap-3">
                                                                                        <div class="flex items-center gap-2 min-w-0">
                                                                                            <div class=if is_curr {
                                                                                                "w-6 h-6 rounded bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shrink-0"
                                                                                            } else {
                                                                                                "w-6 h-6 rounded bg-muted text-muted-foreground flex items-center justify-center shrink-0"
                                                                                            }>
                                                                                                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                                                                    <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                                                                                                    <line x1="8" y1="21" x2="16" y2="21" />
                                                                                                </svg>
                                                                                            </div>
                                                                                            <div class="min-w-0">
                                                                                                <div class="flex items-center gap-1.5">
                                                                                                    <span class="font-medium text-foreground truncate text-xs">{dev.device_name.clone()}</span>
                                                                                                    {if is_curr {
                                                                                                        view! {
                                                                                                            <Badge variant=BadgeVariant::Success class="text-[10px] py-0 px-1.5 font-mono shrink-0 whitespace-nowrap">
                                                                                                                {t_or("supg005.table.this_device", "เครื่องนี้")}
                                                                                                            </Badge>
                                                                                                        }.into_any()
                                                                                                    } else {
                                                                                                        view! { <span /> }.into_any()
                                                                                                    }}
                                                                                                </div>
                                                                                                <div class="text-[11px] text-muted-foreground truncate">
                                                                                                    {format!("{} • {}", dev.ip_address, dev.location)}
                                                                                                </div>
                                                                                            </div>
                                                                                        </div>

                                                                                        <PermissionGate program="SUPG005" action=Action::Approve>
                                                                                        <Button
                                                                                            variant=ButtonVariant::Outline
                                                                                            size=ButtonSize::Sm
                                                                                            class="text-xs h-7 px-2 text-destructive hover:bg-destructive/10 hover:border-destructive/30 shrink-0 gap-1"
                                                                                            on_click=Callback::new({
                                                                                                let (d_uid, dev_id) = (d_uid.clone(), dev_id.clone());
                                                                                                move |_| {
                                                                                                    ctx.revoke_device.run((d_uid.clone(), dev_id.clone()));
                                                                                                }
                                                                                            })
                                                                                        >
                                                                                            <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                                                                                                <line x1="18" y1="6" x2="6" y2="18" />
                                                                                                <line x1="6" y1="6" x2="18" y2="18" />
                                                                                            </svg>
                                                                                            <span>{t_or("supg005.devices.revoke", "เพิกถอน")}</span>
                                                                                        </Button>
                                                                                        </PermissionGate>
                                                                                    </div>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="text-[11px] text-muted-foreground/70 italic">
                                                                            {t_or("supg005.table.no_devices", "ไม่มีอุปกรณ์ล็อกอินอยู่")}
                                                                        </span>
                                                                    }.into_any()
                                                                }}
                                                            </div>
                                                        </TableCell>

                                                        // 6. Action: Suspend / Reactivate Account
                                                        <TableCell class="text-right">
                                                            {if is_self {
                                                                view! {
                                                                    <span class="text-xs text-muted-foreground/60 italic">{t_or("supg005.table.your_account", "บัญชีของคุณ")}</span>
                                                                }.into_any()
                                                            } else if is_suspended {
                                                                let uid_btn = uid_for_revoke.clone();
                                                                view! {
                                                                    <PermissionGate program="SUPG005" action=Action::Write>
                                                                    <ConfirmButton
                                                                        label=Signal::derive(move || t_or("supg005.table.reactivate", "เปิดใช้งานอีกครั้ง"))
                                                                        confirm_text=Signal::derive(move || t_or("supg005.table.reactivate_confirm", "เปิดใช้งานบัญชีนี้อีกครั้ง?"))
                                                                        class="text-xs h-8 ml-auto whitespace-nowrap shadow-sm"
                                                                        on_confirm=Callback::new({
                                                                            let uid_btn = uid_btn.clone();
                                                                            move |_| {
                                                                                ctx.reactivate_user.run(uid_btn.clone());
                                                                            }
                                                                        })
                                                                    />
                                                                    </PermissionGate>
                                                                }.into_any()
                                                            } else {
                                                                let uid_btn = uid_for_revoke.clone();
                                                                view! {
                                                                    <PermissionGate program="SUPG005" action=Action::Write>
                                                                    <ConfirmButton
                                                                        label=Signal::derive(move || t_or("supg005.table.suspend", "ระงับบัญชี"))
                                                                        confirm_text=Signal::derive(move || t_or("supg005.table.suspend_confirm", "ระงับบัญชีนี้และเพิกถอนเซสชันทั้งหมด?"))
                                                                        confirm_label=Signal::derive(move || t_or("supg005.table.suspend_do", "ยืนยันระงับ"))
                                                                        variant=ButtonVariant::Destructive
                                                                        class="text-xs h-8 ml-auto whitespace-nowrap shadow-sm"
                                                                        destructive=true
                                                                        on_confirm=Callback::new({
                                                                            let uid_btn = uid_btn.clone();
                                                                            move |_| {
                                                                                ctx.suspend_user.run(uid_btn.clone());
                                                                            }
                                                                        })
                                                                    />
                                                                    </PermissionGate>
                                                                }.into_any()
                                                            }}
                                                        </TableCell>
                                                    </TableRow>
                                                }
                                            }).collect_view()
                }}
            </DataTable>
        </div>
    }
}
