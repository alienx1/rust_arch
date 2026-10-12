//! DBPG002 Branch Screen (`/db/dbpg002/branch[/:id]`): create / edit a branch.
//! `?company_id=` pre-selects the company.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg002::components::BranchForm;
use crate::pages::db::dbpg002::providers::provide_dbpg002_branch_form_context;

#[component]
pub fn Dbpg002BranchScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let default_company_id = use_query_map().get_untracked().get("company_id");
    let is_edit = id.is_some();
    provide_dbpg002_branch_form_context(id, default_company_id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit {
                            t_or("dbpg002.branch.screen.title_edit", "แก้ไขข้อมูลสาขา (Branch)")
                        } else {
                            t_or("dbpg002.branch.screen.title_create", "เพิ่มสาขาใหม่ (Create Branch)")
                        }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg002.branch.screen.description", "กำหนดข้อมูลสาขา อ้างอิงตาราง db_branch และ db_branch_label • รหัสโปรแกรม: DBPG002")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <BranchForm />
            </PageContent>
        </PageContainer>
    }
}
