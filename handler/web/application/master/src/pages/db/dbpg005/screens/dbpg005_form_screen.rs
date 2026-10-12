//! DBPG005 Form Screen (`/db/dbpg005/:level[/:id]`): create / edit a province, district,
//! sub-district or postal code. `?province_id=` / `?district_id=` pre-select the parents.

use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::NotFoundPage;
use crate::pages::db::dbpg005::components::GeoForm;
use crate::pages::db::dbpg005::models::GeoLevel;
use crate::pages::db::dbpg005::providers::provide_dbpg005_form_context;

#[component]
pub fn Dbpg005FormScreen() -> impl IntoView {
    let params = use_params_map().get_untracked();
    let query = use_query_map().get_untracked();

    let Some(level) = params.get("level").as_deref().and_then(GeoLevel::from_slug) else {
        return view! { <NotFoundPage /> }.into_any();
    };
    let id = params.get("id");
    let is_edit = id.is_some();
    provide_dbpg005_form_context(level, id, query.get("province_id"), query.get("district_id"));

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_fmt("dbpg005.form.title_edit", "แก้ไข{level}", &[("level", level.label().as_str())]) } else { t_fmt("dbpg005.form.title_create", "เพิ่ม{level}", &[("level", level.label().as_str())]) }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("dbpg005.form.description", "กำหนดข้อมูลเขตการปกครองและรหัสไปรษณีย์ • รหัสโปรแกรม: DBPG005")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <GeoForm />
            </PageContent>
        </PageContainer>
    }
    .into_any()
}
