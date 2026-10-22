use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Input(
    #[prop(optional, into)] r#type: Option<String>,
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let input_type = r#type.unwrap_or_else(|| "text".to_string());
    let base = "flex h-9 w-full rounded-md border border-input bg-card px-3 py-1 text-sm shadow-xs transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/30 focus-visible:border-ring focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";
    let merged_class = tw_merge!(base, &class);

    view! {
        <input
            type=input_type
            class=merged_class
            placeholder=placeholder
            disabled=disabled
            readonly=readonly
            prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
            on:input=move |ev| {
                if let Some(cb) = &on_input {
                    cb.run(event_target_value(&ev));
                }
            }
        />
    }
}

#[component]
pub fn SearchInput(
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let base = "flex h-9 w-full rounded-md border border-input bg-card pl-9 pr-3 py-1 text-sm shadow-xs transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/30 focus-visible:border-ring focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";
    let merged_class = tw_merge!(base, &class);
    let input_placeholder = placeholder.unwrap_or_else(|| "Search...".to_string());

    view! {
        <div class="relative flex items-center w-full">
            <span class="absolute left-3 text-muted-foreground pointer-events-none flex items-center">
                <svg
                    class="h-4 w-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
            </span>
            <input
                type="search"
                class=merged_class
                placeholder=input_placeholder
                disabled=disabled
                prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
                on:input=move |ev| {
                    if let Some(cb) = &on_input {
                        cb.run(event_target_value(&ev));
                    }
                }
            />
        </div>
    }
}
