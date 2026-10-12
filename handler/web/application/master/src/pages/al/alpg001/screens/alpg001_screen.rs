//! ALPG001 Screen (`/al/alpg001`): process monitor with filters and the reprocess action.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::al::alpg001::components::ProcessTable;
use crate::pages::al::alpg001::providers::provide_alpg001_context;

#[component]
pub fn Alpg001Screen() -> impl IntoView {
    provide_alpg001_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("alpg001.title", "ติดตามกระบวนการ (Process Monitor)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("alpg001.description", "ดูว่างานเบื้องหลังที่ส่งผ่าน Redpanda สำเร็จหรือไม่ และสั่งประมวลผลซ้ำได้ • รหัสโปรแกรม: ALPG001")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <ProcessTable />
            </PageContent>
        </PageContainer>
    }
}
