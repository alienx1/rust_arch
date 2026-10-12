//! DBPG003 Employee Screen (`/db/dbpg003/employee[/:id]`): create / edit an employee.
//! `?company_id=` pre-selects the company.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg003::components::EmployeeForm;
use crate::pages::db::dbpg003::providers::provide_dbpg003_form_context;

#[component]
pub fn Dbpg003EmployeeScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let default_company_id = use_query_map().get_untracked().get("company_id");
    let is_edit = id.is_some();
    provide_dbpg003_form_context(id, default_company_id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit {
                            t_or("dbpg003.screen.title_edit", "แก้ไขข้อมูลพนักงาน (Employee)")
                        } else {
                            t_or("dbpg003.screen.title_create", "เพิ่มพนักงานใหม่ (Create Employee)")
                        }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg003.screen.description", "กำหนดข้อมูลพนักงาน อ้างอิงตาราง db_employee, db_person_name และ db_person_name_label • รหัสโปรแกรม: DBPG003")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <EmployeeForm />
            </PageContent>
        </PageContainer>
    }
}
