//! NTPG000 Screen (`/nt/ntpg000`): all notifications of the user.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::nt::ntpg000::components::NotificationTable;
use crate::pages::nt::ntpg000::providers::provide_ntpg000_context;

#[component]
pub fn Ntpg000Screen() -> impl IntoView {
    provide_ntpg000_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("ntpg000.title", "การแจ้งเตือน (Notifications)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("ntpg000.description", "สิ่งที่ระบบแจ้งให้คุณทราบ ส่งถึงทุกหน้าจอที่เปิดอยู่ผ่านไอคอนกระดิ่ง • รหัสโปรแกรม: NTPG000")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <NotificationTable />
            </PageContent>
        </PageContainer>
    }
}
