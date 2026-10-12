//! DBPG006 Label Screen (`/db/dbpg006/label`): create / edit an entry.
//! `?category=&key=` edits that entry; `?category=` alone pre-selects the category when creating.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use web_package::core::i18n::t_or;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::db::dbpg006::components::LabelForm;
use crate::pages::db::dbpg006::providers::provide_dbpg006_form_context;

#[component]
pub fn Dbpg006LabelScreen() -> impl IntoView {
    let query = use_query_map().get_untracked();
    let category = query.get("category").filter(|c| !c.is_empty());
    let key = query.get("key").filter(|k| !k.is_empty());
    let target = category.clone().zip(key);
    let is_edit = target.is_some();
    provide_dbpg006_form_context(target, category);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_or("dbpg006.form.title_edit", "แก้ไขข้อความแปลภาษา (Edit Label)") } else { t_or("dbpg006.form.title_create", "เพิ่มข้อความแปลภาษา (Create Label)") }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg006.form.description", "กำหนดข้อความแปลภาษา อ้างอิงตาราง db_label • รหัสโปรแกรม: DBPG006")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <LabelForm />
            </PageContent>
        </PageContainer>
    }
}
