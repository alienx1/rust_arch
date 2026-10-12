//! Language Switcher UI widget: allows users to toggle/select application language.

use leptos::prelude::*;
use super::i18n_context::use_i18n;

#[component]
pub fn LanguageSwitcher(
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let class_str = class.unwrap_or_default();

    let toggle_language = move |_| {
        let current = i18n.current_lang.get();
        let next = if current == "th" { "en" } else { "th" };
        i18n.set_lang.run(next.to_string());
    };

    view! {
        <button
            type="button"
            class=format!(
                "inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-md border border-border bg-background hover:bg-muted text-foreground transition-all duration-150 cursor-pointer shadow-xs {}",
                class_str
            )
            on:click=toggle_language
            title=move || if i18n.current_lang.get() == "th" { "Switch to English" } else { "เปลี่ยนเป็นภาษาไทย" }
            aria-label="Switch Language"
        >
            <svg class="h-3.5 w-3.5 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <span class="uppercase font-bold tracking-wider text-[11px] text-foreground">
                {move || i18n.current_lang.get()}
            </span>
        </button>
    }
}
