//! ALPG000 Screen (`/al/alpg000`): audit log list with filters.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::al::alpg000::components::AuditLogTable;
use crate::pages::al::alpg000::providers::provide_alpg000_context;

#[component]
pub fn Alpg000Screen() -> impl IntoView {
    provide_alpg000_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("alpg000.title", "บันทึกเหตุการณ์ (Audit Log)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("alpg000.description", "ตรวจสอบการเข้าใช้งานของผู้ใช้และบันทึกระบบของทุก API (คำขอ คำตอบ สถานะ ส่วนหัว) • รหัสโปรแกรม: ALPG000")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <AuditLogTable />
            </PageContent>
        </PageContainer>
    }
}
