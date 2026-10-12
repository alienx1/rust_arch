//! IAM Session Reactive Context & State Provider

use web_package::core::i18n::{t_or, t_fmt};
use crate::pages::su::supg005::models::{
    Supg005SessionSummary, PaginatedResult, UserSessionOverviewDto,
};
use crate::pages::su::supg005::services::Supg005Service;
use leptos::prelude::*;
use web_package::core::auth::use_auth;

/// Shared reactive state context for IAM Session screen
#[derive(Clone, Copy)]
pub struct Supg005Context {
    pub data: ReadSignal<Option<Supg005SessionSummary>>,
    pub is_loading: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub active_tab: Signal<String>,
    pub search_query: Signal<String>,
    pub module_filter: Signal<String>,
    pub user_search_query: Signal<String>,
    /// Account state filter: `ALL` / `ACTIVE` / `SUSPENDED`
    pub account_status_filter: Signal<String>,
    pub current_page: Signal<u64>,
    pub page_size: Signal<u64>,
    /// Rows per page chosen in the table footer (returns to page 1)
    pub set_limit: Callback<u64>,
    pub paginated_users: ReadSignal<Option<PaginatedResult<UserSessionOverviewDto>>>,
    pub is_users_loading: ReadSignal<bool>,
    pub action_notification: Signal<Option<String>>,
    pub set_active_tab: WriteSignal<String>,
    pub set_search_query: WriteSignal<String>,
    pub set_module_filter: WriteSignal<String>,
    pub set_user_search_query: WriteSignal<String>,
    pub set_account_status_filter: WriteSignal<String>,
    pub set_current_page: WriteSignal<u64>,
    pub set_action_notification: WriteSignal<Option<String>>,
    pub refresh: Callback<()>,
    pub refresh_users: Callback<()>,
    pub switch_company: Callback<String>,
    pub revoke_other_sessions: Callback<()>,
    pub suspend_user: Callback<String>,
    pub reactivate_user: Callback<String>,
    pub revoke_device: Callback<(String, String)>,
}

/// Provides Supg005Context to the Leptos reactive tree.
pub fn provide_supg005_context() -> Supg005Context {
    let auth = use_auth();
    let (data, set_data) = signal(Option::<Supg005SessionSummary>::None);
    let (is_loading, set_is_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (refresh_trigger, set_refresh_trigger) = signal(0u32);

    let (active_tab_raw, set_active_tab) = signal("all_users".to_string());
    let active_tab = Signal::derive(move || active_tab_raw.get());

    let (search_query_raw, set_search_query) = signal("".to_string());
    let search_query = Signal::derive(move || search_query_raw.get());

    let (module_filter_raw, set_module_filter) = signal("ALL".to_string());
    let module_filter = Signal::derive(move || module_filter_raw.get());

    let (user_search_query_raw, set_user_search_query) = signal("".to_string());
    let user_search_query = Signal::derive(move || user_search_query_raw.get());

    let (account_status_filter_raw, set_account_status_filter) = signal("ALL".to_string());
    let account_status_filter = Signal::derive(move || account_status_filter_raw.get());

    let (current_page_raw, set_current_page) = signal(1u64);
    let current_page = Signal::derive(move || current_page_raw.get());

    let (page_size_raw, set_page_size) = signal(5u64);
    let page_size = Signal::derive(move || page_size_raw.get());

    let (paginated_users, set_paginated_users) =
        signal(Option::<PaginatedResult<UserSessionOverviewDto>>::None);
    let (is_users_loading, set_is_users_loading) = signal(true);
    let (users_refresh_trigger, set_users_refresh_trigger) = signal(0u32);

    let (action_notification_raw, set_action_notification) = signal(Option::<String>::None);
    let action_notification = Signal::derive(move || action_notification_raw.get());

    let service = Supg005Service::new();
    let effect_service = service.clone();

    // track เฉพาะ session id / active — ไม่ reload ทั้งหน้าเมื่อ heartbeat ได้ session เดิม
    let session_key = Memo::new(move |_| auth.session.with(|s| s.as_ref().map(|s| (s.id.clone(), s.active))));

    // Effect โหลดข้อมูลเซสชันหลักเมื่อ auth state เปลี่ยน หรือ refresh trigger ถูกเรียก
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let _ = session_key.get();
        let kratos_session = auth.session.get_untracked();
        let svc = effect_service.clone();

        set_is_loading.set(true);
        set_error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            match svc.fetch_session_summary(kratos_session).await {
                Ok(summary) => {
                    set_data.set(Some(summary));
                }
                Err(e) => {
                    set_error.set(Some(t_fmt("supg005.ctx.load_failed", "ไม่สามารถโหลดข้อมูลเซสชันได้: {error}", &[("error", &e.to_string())])));
                }
            }
            set_is_loading.set(false);
        });
    });

    let users_service = service.clone();

    // Effect โหลดข้อมูลผู้ใช้งานแบบ Server-Side SQL LIMIT & OFFSET เมื่อ page, search, status หรือ refresh เปลี่ยน
    Effect::new(move |_| {
        let p = current_page.get();
        let limit = page_size.get();
        let q = user_search_query.get();
        let account_st = account_status_filter.get();
        let _ = users_refresh_trigger.get();
        let current_session_id = auth.session.get_untracked().map(|s| s.id);
        let svc = users_service.clone();

        set_is_users_loading.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            let res = svc
                .fetch_paginated_users(p, limit, Some(q), None, Some(account_st), current_session_id)
                .await;
            match res {
                Ok(data) => {
                    set_paginated_users.set(Some(data));
                }
                Err(e) => {
                    leptos::logging::error!("Error fetching paginated users: {e}");
                }
            }
            set_is_users_loading.set(false);
        });
    });

    let refresh = Callback::new(move |()| {
        auth.check_session();
        set_refresh_trigger.update(|n| *n += 1);
        set_users_refresh_trigger.update(|n| *n += 1);
        set_action_notification.set(Some(t_or("supg005.ctx.refreshed", "รีเฟรชข้อมูลเซสชันสำเร็จ")));
    });

    let refresh_users = Callback::new(move |()| {
        set_users_refresh_trigger.update(|n| *n += 1);
    });

    let switch_company = Callback::new(move |company_code: String| {
        set_data.update(|current| {
            if let Some(summary) = current.as_mut() {
                if let Some(target) = summary
                    .scope
                    .available_companies
                    .iter()
                    .find(|c| c.code == company_code || c.id == company_code)
                {
                    summary.scope.company_id = target.id.clone();
                    summary.scope.company_code = target.code.clone();
                    summary.scope.company_name = target.name.clone();
                    set_action_notification.set(Some(t_fmt("supg005.ctx.switched", "สลับไปยัง [{code}] {name}", &[("code", &target.code.to_string()),("name", &target.name.to_string())])));
                }
            }
        });
    });

    // ทุก action: เรียก API ก่อน → สำเร็จแล้วค่อย refetch จาก server (ไม่ใช้ optimistic update)
    // → ล้มเหลวให้แจ้ง error และไม่แตะข้อมูลที่แสดงอยู่
    let revoke_other_sessions = {
        let svc = service.clone();
        Callback::new(move |()| {
            let api_svc = svc.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api_svc.revoke_other_sessions().await {
                    Ok(()) => {
                        set_action_notification
                            .set(Some(t_or("supg005.ctx.revoked_others", "เพิกถอนเซสชันอุปกรณ์อื่นของบัญชีนี้ทั้งหมดเรียบร้อยแล้ว")));
                        set_refresh_trigger.update(|n| *n += 1);
                        set_users_refresh_trigger.update(|n| *n += 1);
                    }
                    Err(e) => {
                        leptos::logging::error!("revoke_other_sessions failed: {e}");
                        set_action_notification.set(Some(t_fmt("supg005.ctx.revoke_failed", "เพิกถอนไม่สำเร็จ: {error}", &[("error", &e.to_string())])));
                    }
                }
            });
        })
    };

    // ระงับ / เปิดใช้งานบัญชีผู้ใช้ (Suspend / Reactivate Account)
    let make_account_toggle = |suspend: bool| {
        let svc = service.clone();
        Callback::new(move |target_user_id: String| {
            let api_svc = svc.clone();
            let target_email = paginated_users.with_untracked(|p| {
                p.as_ref().and_then(|p| {
                    p.items.iter().find(|u| u.user_id == target_user_id).map(|u| u.email.clone())
                })
            });
            let is_self = suspend
                && target_email.is_some()
                && auth.user.get_untracked().and_then(|u| u.email()) == target_email;

            wasm_bindgen_futures::spawn_local(async move {
                if is_self {
                    set_action_notification.set(Some(t_or("supg005.ctx.cannot_suspend_self", "ไม่สามารถระงับบัญชีของตนเองได้")));
                    return;
                }
                let res = if suspend {
                    api_svc.suspend_user(&target_user_id).await
                } else {
                    api_svc.reactivate_user(&target_user_id).await
                };
                match res {
                    Ok(()) => {
                        set_action_notification.set(Some(if suspend {
                            t_or("supg005.ctx.suspended", "ระงับบัญชีและเพิกถอนทุกเซสชันของผู้ใช้สำเร็จ")
                        } else {
                            t_or("supg005.ctx.reactivated", "เปิดใช้งานบัญชีผู้ใช้อีกครั้งสำเร็จ")
                        }));
                        set_refresh_trigger.update(|n| *n += 1);
                        set_users_refresh_trigger.update(|n| *n += 1);
                    }
                    Err(e) => {
                        leptos::logging::error!("account toggle failed: {e}");
                        set_action_notification.set(Some(t_fmt("supg005.ctx.action_failed", "ดำเนินการไม่สำเร็จ: {error}", &[("error", &e.to_string())])));
                    }
                }
            });
        })
    };
    let suspend_user = make_account_toggle(true);
    let reactivate_user = make_account_toggle(false);

    // เพิกถอนเซสชันอุปกรณ์ที่ระบุ (Revoke Specific Device Session)
    let revoke_device = {
        let svc = service.clone();
        Callback::new(move |(_target_user_id, target_device_id): (String, String)| {
            let api_svc = svc.clone();

            let is_current_session = auth
                .session
                .get_untracked()
                .map(|s| s.id == target_device_id)
                .unwrap_or(false);

            wasm_bindgen_futures::spawn_local(async move {
                match api_svc.revoke_device_session(&target_device_id).await {
                    Ok(()) => {
                        set_action_notification
                            .set(Some(t_or("supg005.ctx.revoked_device", "เพิกถอนเซสชันอุปกรณ์เรียบร้อยแล้ว")));
                        if is_current_session {
                            auth.logout();
                        } else {
                            set_refresh_trigger.update(|n| *n += 1);
                            set_users_refresh_trigger.update(|n| *n += 1);
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("revoke_device_session failed: {e}");
                        set_action_notification.set(Some(t_fmt("supg005.ctx.revoke_failed", "เพิกถอนไม่สำเร็จ: {error}", &[("error", &e.to_string())])));
                    }
                }
            });
        })
    };

    let ctx = Supg005Context {
        data,
        is_loading,
        error,
        active_tab,
        search_query,
        module_filter,
        user_search_query,
        account_status_filter,
        current_page,
        page_size,
        set_limit: Callback::new(move |n: u64| {
            set_page_size.set(n.clamp(1, 100));
            set_current_page.set(1);
        }),
        paginated_users,
        is_users_loading,
        action_notification,
        set_active_tab,
        set_search_query,
        set_module_filter,
        set_user_search_query,
        set_account_status_filter,
        set_current_page,
        set_action_notification,
        refresh,
        refresh_users,
        switch_company,
        revoke_other_sessions,
        suspend_user,
        reactivate_user,
        revoke_device,
    };

    provide_context(ctx);
    ctx
}

/// Access Supg005Context in child components
pub fn use_supg005() -> Supg005Context {
    expect_context::<Supg005Context>()
}
