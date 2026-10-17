//! Demo Screen Component (Static UI Component Showcase)
//!
//! Provides a comprehensive live showcase of all UI components in the design system.
//! Only visible and accessible when running in localhost or development environments.
//! Rendered inside `MainLayout`'s `<Outlet/>`; the sidebar/topbar shell is persistent.

use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use web_package::layout::{
    PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle,
};
use web_package::ui::component::{Badge, BadgeVariant, Button, ButtonSize, ButtonVariant};

use crate::pages::demo::components::{
    ButtonShowcase, DataShowcase, FeedbackShowcase, FormShowcase, LayerShowcase,
    NavigationShowcase, OverlayShowcase,
};
use crate::pages::demo::providers::provide_demo_context;
use crate::pages::demo::services::is_dev_env;

#[component]
pub fn DemoScreen() -> impl IntoView {
    let navigate = use_navigate();
    let go_home = move |_: ()| navigate("/", NavigateOptions::default());

    // Security & Environment Gate: Only allow access in dev/localhost
    if !is_dev_env() {
        Effect::new(move |_| go_home(()));
        return view! {
            <div class="min-h-[60vh] flex flex-col items-center justify-center p-6 text-center space-y-3">
                <Badge variant=BadgeVariant::Destructive>"Access Restricted"</Badge>
                <h2 class="text-xl font-bold text-foreground">"หน้าสาธิตเปิดเฉพาะในโหมด Local Dev เท่านั้น"</h2>
                <p class="text-xs text-muted-foreground">"This page is disabled in production environments."</p>
                <a href="/" class="text-xs text-primary hover:underline font-medium">"กลับสู่หน้าหลัก"</a>
            </div>
        }.into_any();
    }

    let ctx = provide_demo_context();
    let (active_tab, set_active_tab) = ctx.active_tab;

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <div class="flex items-center gap-2">
                        <PageTitle>"แคตตาล็อกคอมโพเนนต์ส่วนติดต่อผู้ใช้ (UI Components Demo)"</PageTitle>
                        <Badge variant=BadgeVariant::Success class="text-[10px]">"Static Mode"</Badge>
                    </div>
                    <PageDescription>
                        "แสดงผลคอมโพเนนต์ทั้งหมดของแพลตฟอร์มแบบ Interactive: Buttons, Forms, Alerts, Tables, Cards, Modals"
                    </PageDescription>
                </div>
                <PageActions>
                    <Button
                        variant=ButtonVariant::Outline
                        class="text-xs h-9 gap-1.5"
                        on_click=Callback::new(move |_| go_home(()))
                    >
                        "กลับหน้าแดชบอร์ดหลัก"
                    </Button>
                </PageActions>
            </PageHeader>

            <PageContent>
                <div class="space-y-6 pb-12">
                    // Category Filter Bar
                    <div class="flex flex-wrap gap-2 pb-2 border-b border-border">
                        {move || {
                            let current = active_tab.get();
                            [
                            ("overview", "ทั้งหมด (Overview)"),
                            ("buttons", "1. Buttons"),
                            ("forms", "2. Forms"),
                            ("feedback", "3. Feedback"),
                            ("data", "4. Data Display"),
                            ("overlays", "5. Overlays"),
                            ("navigation", "6. Navigation"),
                            ("layers", "7. Layers"),
                        ].into_iter().map(|(val, lbl)| {
                            let val_for_click = val.to_string();
                            let is_active = current == val;
                            view! {
                                <Button
                                    size=ButtonSize::Sm
                                    variant=if is_active { ButtonVariant::Default } else { ButtonVariant::Outline }
                                    class="text-xs"
                                    on_click=Callback::new(move |_| set_active_tab.set(val_for_click.clone()))
                                >
                                    {lbl}
                                </Button>
                            }
                        }).collect::<Vec<_>>()
                        }}
                    </div>

                    // Filtered Showcase Sections
                    {move || {
                        let tab = active_tab.get();
                        view! {
                            <div class="space-y-6">
                                {(tab == "overview" || tab == "buttons").then(|| view! { <ButtonShowcase /> })}
                                {(tab == "overview" || tab == "forms").then(|| view! { <FormShowcase /> })}
                                {(tab == "overview" || tab == "feedback").then(|| view! { <FeedbackShowcase /> })}
                                {(tab == "overview" || tab == "data").then(|| view! { <DataShowcase /> })}
                                {(tab == "overview" || tab == "overlays").then(|| view! { <OverlayShowcase /> })}
                                {(tab == "overview" || tab == "navigation").then(|| view! { <NavigationShowcase /> })}
                                {(tab == "overview" || tab == "layers").then(|| view! { <LayerShowcase /> })}
                            </div>
                        }
                    }}
                </div>
            </PageContent>
        </PageContainer>
    }.into_any()
}
