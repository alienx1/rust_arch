use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Textarea(
    #[prop(optional)] value: Option<Signal<String>>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] readonly: bool,
    #[prop(default = 3)] rows: u32,
    #[prop(optional)] on_input: Option<Callback<String>>,
) -> impl IntoView {
    let base = "flex min-h-[60px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50";
    let merged_class = tw_merge!(base, &class);

    view! {
        <textarea
            class=merged_class
            placeholder=placeholder
            disabled=disabled
            readonly=readonly
            rows=rows
            prop:value=move || value.as_ref().map(|v| v.get()).unwrap_or_default()
            on:input=move |ev| {
                if let Some(cb) = &on_input {
                    cb.run(event_target_value(&ev));
                }
            }
        />
    }
}
