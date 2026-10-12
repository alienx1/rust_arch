//! SUPG004 User Screen (`/su/supg004/user[/:id]`): create an account / change group or status.

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::su::supg004::components::UserForm;
use crate::pages::su::supg004::providers::provide_supg004_form_context;

#[component]
pub fn Supg004UserScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let is_edit = id.is_some();
    provide_supg004_form_context(id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_or("supg004.user.title_edit", "แก้ไขบัญชีผู้ใช้ (User Account)") } else { t_or("supg004.user.title_create", "เพิ่มบัญชีผู้ใช้ใหม่ (Create User)") }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("supg004.user.description", "ผูกบัญชีกับพนักงานและกลุ่มสิทธิ์ อ้างอิงตาราง su_user • รหัสโปรแกรม: SUPG004")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <UserForm />
            </PageContent>
        </PageContainer>
    }
}
