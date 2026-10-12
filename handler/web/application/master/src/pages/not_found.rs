use leptos::prelude::*;
use web_package::core::i18n::t_or;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-[60vh] h-full flex flex-col items-center justify-center p-6 text-center space-y-4">
            <h1 class="text-4xl font-extrabold text-foreground">{move || t_or("common.not_found.title", "404")}</h1>
            <p class="text-muted-foreground text-sm">{move || t_or("common.not_found.message", "ไม่พบหน้าที่คุณต้องการ")}</p>
            <a href="/" class="text-sm text-primary hover:underline font-medium">
                {move || t_or("common.action.back_home", "กลับสู่หน้าหลัก")}
            </a>
        </div>
    }
}
