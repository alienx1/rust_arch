//! DBPG001 Company Screen (`/db/dbpg001/company[/:id]`)
//!
//! Page layout for creating / editing a company. `?organization_id=` pre-selects the parent.

use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg001::components::CompanyForm;
use crate::pages::db::dbpg001::providers::provide_dbpg001_company_context;

#[component]
pub fn Dbpg001CompanyScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let default_org_id = use_query_map().get_untracked().get("organization_id");
    let is_edit = id.is_some();
    provide_dbpg001_company_context(id, default_org_id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {move || if is_edit {
                            t_or("dbpg001.company.dialog.edit_title", "แก้ไขข้อมูลบริษัทในเครือ (Company)")
                        } else {
                            t_or("dbpg001.company.dialog.create_title", "เพิ่มบริษัทใหม่ (Create Company)")
                        }}
                    </PageTitle>
                    <PageDescription>
                        {move || t_or("dbpg001.company.description", "กำหนดข้อมูลนิติบุคคลและบริษัทในเครือ อ้างอิงตาราง db_company และ db_company_label • รหัสโปรแกรม: DBPG001")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <CompanyForm />
            </PageContent>
        </PageContainer>
    }
}
