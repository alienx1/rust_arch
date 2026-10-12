//! SUPG003 Matrix Screen (`/su/supg003/group/:id/matrix`): edit a group's permission matrix.

use web_package::core::i18n::{t_or, t_fmt};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{Button, ButtonVariant};

use crate::pages::NotFoundPage;
use crate::pages::su::supg003::components::MatrixTable;
use crate::pages::su::supg003::providers::provide_supg003_matrix_context;

#[component]
pub fn Supg003MatrixScreen() -> impl IntoView {
    let Some(id) = use_params_map().get_untracked().get("id") else {
        return view! { <NotFoundPage /> }.into_any();
    };
    let ctx = provide_supg003_matrix_context(id);

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>
                        {move || match ctx.group.get() {
                            Some(g) => t_fmt("supg003.matrix.title_with_code", "เมทริกซ์สิทธิ์: {code}", &[("code", &g.code.to_string())]),
                            None => t_or("supg003.matrix.title", "เมทริกซ์สิทธิ์"),
                        }}
                    </PageTitle>
                    <PageDescription>
                        {move || ctx.group.get().map(|g| t_fmt("supg003.matrix.subtitle", "ผู้ใช้ {count} คน • รหัสโปรแกรม: SUPG003", &[("count", &g.user_count.to_string())]))}
                    </PageDescription>
                </div>
                <PageActions>
                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9"
                        on_click=Callback::new(move |_| ctx.back.run(()))
                    >
                        {t_or("supg003.matrix.back", "← กลับรายการกลุ่ม")}
                    </Button>
                </PageActions>
            </PageHeader>
            <PageContent>
                <MatrixTable />
            </PageContent>
        </PageContainer>
    }
    .into_any()
}
