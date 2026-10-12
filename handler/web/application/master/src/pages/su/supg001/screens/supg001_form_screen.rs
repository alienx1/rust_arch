//! SUPG001 Form Screen (`/su/supg001/:kind[/:id]`): create / edit a system or module.
//! `?system_id=` pre-selects the system of a new module.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::NotFoundPage;
use crate::pages::su::supg001::components::UnitForm;
use crate::pages::su::supg001::models::UnitKind;
use crate::pages::su::supg001::providers::provide_supg001_form_context;

#[component]
pub fn Supg001FormScreen() -> impl IntoView {
    let params = use_params_map().get_untracked();
    let query = use_query_map().get_untracked();

    let Some(kind) = params.get("kind").as_deref().and_then(UnitKind::from_slug) else {
        return view! { <NotFoundPage /> }.into_any();
    };
    let id = params.get("id");
    let is_edit = id.is_some();
    provide_supg001_form_context(kind, id, query.get("system_id"));

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit {
                            t_fmt("supg001.form.title_edit", "แก้ไข{kind}", &[("kind", &kind.label())])
                        } else {
                            t_fmt("supg001.form.title_create", "เพิ่ม{kind}", &[("kind", &kind.label())])
                        }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("supg001.form.description", "กำหนดข้อมูลระบบ/โมดูล อ้างอิงตาราง db_system, db_module และตาราง label • รหัสโปรแกรม: SUPG001")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <UnitForm />
            </PageContent>
        </PageContainer>
    }
    .into_any()
}
