use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectItem {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

#[component]
pub fn Select(
    #[prop(optional, into)] value: Option<Signal<String>>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
    #[prop(optional, into)] on_change: Option<Callback<String>>,
    children: Children,
) -> impl IntoView {
    let base = "flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 cursor-pointer";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class="relative w-full">
            <select
                class=merged_class
                disabled=disabled
                prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
                on:change=move |ev| {
                    if let Some(cb) = &on_change {
                        cb.run(event_target_value(&ev));
                    }
                }
            >
                {children()}
            </select>
        </div>
    }
}

#[component]
pub fn SelectOption(
    #[prop(into)] value: String,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <option value=value disabled=disabled class="bg-popover text-popover-foreground py-1">
            {children()}
        </option>
    }
}
