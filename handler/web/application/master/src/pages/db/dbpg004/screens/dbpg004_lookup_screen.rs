//! DBPG004 Lookup Screen (`/db/dbpg004/lookup[/:id]`): create / edit a reference value.
//! `?category=` pre-selects the category.

use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg004::components::LookupForm;
use crate::pages::db::dbpg004::providers::provide_dbpg004_form_context;

#[component]
pub fn Dbpg004LookupScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let default_category = use_query_map().get_untracked().get("category");
    let is_edit = id.is_some();
    provide_dbpg004_form_context(id, default_category);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_or("dbpg004.form.title_edit", "แก้ไขค่าอ้างอิง (Lookup)") } else { t_or("dbpg004.form.title_create", "เพิ่มค่าอ้างอิงใหม่ (Create Lookup)") }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg004.form.description", "กำหนดค่าอ้างอิง อ้างอิงตาราง db_master_lookup และ db_master_lookup_label • รหัสโปรแกรม: DBPG004")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <LookupForm />
            </PageContent>
        </PageContainer>
    }
}
