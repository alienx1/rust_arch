//! RPPG000 Screen (`/rp/rppg000`): my reports with preview and download.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::rp::rppg000::components::{ReportJobTable, ReportPreviewDialog};
use crate::pages::rp::rppg000::providers::provide_rppg000_context;

#[component]
pub fn Rppg000Screen() -> impl IntoView {
    provide_rppg000_context();

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("rppg000.title", "รายงานของฉัน (My Reports)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("rppg000.description", "รายงานที่ขอไว้จะถูกสร้างเบื้องหลัง เมื่อพร้อมระบบแจ้งเตือนและให้ดูตัวอย่างหรือดาวน์โหลดได้ • รหัสโปรแกรม: RPPG000")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <ReportJobTable />
                <ReportPreviewDialog />
            </PageContent>
        </PageContainer>
    }
}
