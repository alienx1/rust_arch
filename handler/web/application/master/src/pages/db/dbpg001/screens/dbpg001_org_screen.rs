//! DBPG001 Organization Screen (`/db/dbpg001/org[/:id]`)
//!
//! Page layout for creating / editing an organization.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg001::components::OrgForm;
use crate::pages::db::dbpg001::providers::provide_dbpg001_org_context;

#[component]
pub fn Dbpg001OrgScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let is_edit = id.is_some();
    provide_dbpg001_org_context(id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {move || if is_edit {
                            t_or("dbpg001.org.dialog.edit_title", "แก้ไขข้อมูลองค์กร (Organization)")
                        } else {
                            t_or("dbpg001.org.dialog.create_title", "เพิ่มองค์กรใหม่ (Create Organization)")
                        }}
                    </PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg001.org.description", "กำหนดข้อมูลกลุ่มองค์กรหลัก อ้างอิงตาราง db_organization และ db_organization_label • รหัสโปรแกรม: DBPG001")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <OrgForm />
            </PageContent>
        </PageContainer>
    }
}
