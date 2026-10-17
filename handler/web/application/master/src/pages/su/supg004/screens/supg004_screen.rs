//! SUPG004 Screen (`/su/supg004`): user accounts
//!
//! The account table; Renders the standard
//! layout with an empty table when the API is unreachable (or the account lacks READ on SUPG004).

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{StatusColor, StatusDot};

use crate::pages::su::supg004::components::{UserTable};
use crate::pages::su::supg004::providers::provide_supg004_context;

#[component]
pub fn Supg004Screen() -> impl IntoView {
    let ctx = provide_supg004_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("supg004.title", "บัญชีผู้ใช้ (User Accounts)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg004.description", "สร้างบัญชีเข้าสู่ระบบให้พนักงาน กำหนดกลุ่มสิทธิ์ ระงับ/เปิดใช้งาน และส่งอีเมลตั้งรหัสผ่าน • รหัสโปรแกรม: SUPG004")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || if ctx.error.get().is_some() {
                        view! { <StatusDot color=StatusColor::Destructive label="Master DB: Disconnected" /> }.into_any()
                    } else {
                        view! { <StatusDot color=StatusColor::Success label="Master DB: Connected" /> }.into_any()
                    }}
                </PageActions>
            </PageHeader>

            <PageContent>
                {move || ctx.error.get().map(|err| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm mb-4">
                        <span class="font-medium">{err}</span>
                    </div>
                })}
                <UserTable />
            </PageContent>
        </PageContainer>
    }
}
