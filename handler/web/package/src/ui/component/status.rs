use crate::core::i18n::t_or;
use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum StatusColor {
    #[default]
    Success,
    Warning,
    Destructive,
    Info,
    Muted,
}

impl StatusColor {
    pub fn dot_color(&self) -> &'static str {
        match self {
            Self::Success => "bg-emerald-500",
            Self::Warning => "bg-amber-500",
            Self::Destructive => "bg-destructive",
            Self::Info => "bg-sky-500",
            Self::Muted => "bg-muted-foreground",
        }
    }
}

#[component]
pub fn StatusDot(
    #[prop(optional)] color: StatusColor,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "inline-flex items-center gap-1.5 text-xs font-medium text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <span class=merged_class>
            <span class=format!("h-2 w-2 rounded-full {}", color.dot_color())></span>
            {label}
        </span>
    }
}

/// Standalone Status / Error component styled identically to 404 NotFound page.
/// Used across application pages when backend connection fails or data cannot be loaded.
#[component]
pub fn StatusErrorState(
    #[prop(optional, into)] code: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] message: Option<String>,
    #[prop(optional)] on_retry: Option<Callback<()>>,
) -> impl IntoView {
    let code_str = code.unwrap_or_else(|| "503".to_string());
    let title_str = title.unwrap_or_else(|| t_or("common.state.load_error_title", "ไม่สามารถโหลดข้อมูลได้"));
    let message_str = message.unwrap_or_else(|| {
        t_or(
            "common.state.load_error_message",
            "ไม่สามารถเชื่อมต่อกับบริการข้อมูลหรือระบบหลังบ้านได้ในขณะนี้ กรุณาตรวจสอบการเชื่อมต่อและลองใหม่อีกครั้ง",
        )
    });

    view! {
        <div class="min-h-[60vh] h-full flex flex-col items-center justify-center p-6 text-center space-y-4 animate-in fade-in-50">
            <h1 class="text-4xl font-extrabold text-foreground">{code_str}</h1>
            <p class="text-base font-semibold text-foreground">{title_str}</p>
            <p class="text-muted-foreground text-sm max-w-md">{message_str}</p>
            <div class="flex items-center justify-center gap-3 pt-2">
                {on_retry.map(|cb| view! {
                    <button
                        type="button"
                        class="inline-flex items-center justify-center rounded-md text-xs font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 cursor-pointer shadow-sm"
                        on:click=move |_| cb.run(())
                    >
                        <svg class="h-3.5 w-3.5 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                            <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                            <path d="M3 3v5h5" />
                            <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                            <path d="M16 21h5v-5" />
                        </svg>
                        {move || t_or("common.action.retry", "ลองใหม่อีกครั้ง")}
                    </button>
                })}
                <a href="/" class="inline-flex items-center justify-center rounded-md text-xs font-medium text-primary hover:underline h-9 px-3">
                    {move || t_or("common.action.back_home", "กลับสู่หน้าหลัก")}
                </a>
            </div>
        </div>
    }
}
