//! SUPG002 Program Screen (`/su/supg002/program[/:id]`): create / edit a program.
//! `?system_id=` / `?module_id=` pre-select the placement of a new program.

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use leptos_router::hooks::{use_params_map, use_query_map};
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::su::supg002::components::ProgramForm;
use crate::pages::su::supg002::providers::provide_supg002_form_context;

#[component]
pub fn Supg002ProgramScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let query = use_query_map().get_untracked();
    let is_edit = id.is_some();
    provide_supg002_form_context(id, query.get("system_id"), query.get("module_id"));

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_or("supg002.form.title_edit", "แก้ไขโปรแกรม (Program)") } else { t_or("supg002.form.title_create", "เพิ่มโปรแกรมใหม่ (Create Program)") }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("supg002.form.description", "กำหนดตำแหน่ง เส้นทาง ไอคอน และชื่อเมนู อ้างอิงตาราง db_program และ db_program_label • รหัสโปรแกรม: SUPG002")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <ProgramForm />
            </PageContent>
        </PageContainer>
    }
}
