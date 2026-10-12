//! Light / dark mode switch. The choice is kept in `localStorage` (`theme` = `light` | `dark`);
//! without one the system preference decides. The page's `<html>` gets the `dark` class (Tailwind
//! `darkMode: 'class'`); an inline script in `index.html` applies it before the first paint so the
//! page does not flash.

use crate::core::i18n::t_or;
use leptos::prelude::*;

const STORAGE_KEY: &str = "theme";
const SUN: &str = r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>"#;
const MOON: &str = r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#;

/// Whether `<html>` currently has the `dark` class
fn is_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .is_some_and(|e| e.class_list().contains("dark"))
}

fn apply(dark: bool) {
    if let Some(root) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
        let _ = root.class_list().toggle_with_force("dark", dark);
    }
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.set_item(STORAGE_KEY, if dark { "dark" } else { "light" });
    }
}

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (dark, set_dark) = signal(is_dark());
    let toggle = move |_| {
        let next = !dark.get_untracked();
        apply(next);
        set_dark.set(next);
    };
    let label = move || {
        if dark.get() {
            t_or("common.theme.to_light", "สลับเป็นโหมดสว่าง")
        } else {
            t_or("common.theme.to_dark", "สลับเป็นโหมดมืด")
        }
    };

    view! {
        <button
            type="button"
            class="p-1.5 rounded-md hover:bg-muted text-muted-foreground transition-colors cursor-pointer"
            aria-label=label
            title=label
            aria-pressed=move || dark.get().to_string()
            on:click=toggle
        >
            <svg
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                inner_html=move || if dark.get() { SUN } else { MOON }
            ></svg>
        </button>
    }
}
