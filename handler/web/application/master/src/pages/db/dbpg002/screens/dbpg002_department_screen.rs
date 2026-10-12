//! DBPG002 Department Screen (`/db/dbpg002/department[/:id]`): create / edit a department.
//! `?branch_id=` pre-selects the branch, `?company_id=` narrows the branch selector.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg002::components::DepartmentForm;
use crate::pages::db::dbpg002::providers::provide_dbpg002_department_form_context;

#[component]
pub fn Dbpg002DepartmentScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let query = use_query_map().get_untracked();
    let is_edit = id.is_some();
    provide_dbpg002_department_form_context(id, query.get("company_id"), query.get("branch_id"));

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit {
                            t_or("dbpg002.dept.screen.title_edit", "แก้ไขข้อมูลแผนก (Department)")
                        } else {
                            t_or("dbpg002.dept.screen.title_create", "เพิ่มแผนกใหม่ (Create Department)")
                        }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg002.dept.screen.description", "กำหนดข้อมูลแผนกงาน อ้างอิงตาราง db_department และ db_department_label • รหัสโปรแกรม: DBPG002")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <DepartmentForm />
            </PageContent>
        </PageContainer>
    }
}
