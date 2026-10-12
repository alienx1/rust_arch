//! SUPG003 Group Screen (`/su/supg003/group[/:id]`): create a group / switch it on or off.

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use web_package::layout::{PageContainer, PageContent, PageDescription, PageHeader, PageTitle};

use crate::pages::su::supg003::components::GroupForm;
use crate::pages::su::supg003::providers::provide_supg003_form_context;

#[component]
pub fn Supg003GroupScreen() -> impl IntoView {
    let id = use_params_map().get_untracked().get("id");
    let is_edit = id.is_some();
    provide_supg003_form_context(id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {if is_edit { t_or("supg003.group.title_edit", "แก้ไขกลุ่มสิทธิ์ (Permission Group)") } else { t_or("supg003.group.title_create", "เพิ่มกลุ่มสิทธิ์ใหม่ (Create Group)") }}
                    </PageTitle>
                    <PageDescription>
                        {t_or("supg003.group.description", "กำหนดรหัสกลุ่มและขอบเขตข้อมูล อ้างอิงตาราง su_permission_group • รหัสโปรแกรม: SUPG003")}
                    </PageDescription>
                </div>
            </PageHeader>
            <PageContent>
                <GroupForm />
            </PageContent>
        </PageContainer>
    }
}
