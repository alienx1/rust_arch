use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Checkbox(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let base = "peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 accent-primary cursor-pointer";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class="flex items-center space-x-2">
            <input
                type="checkbox"
                class=merged_class
                disabled=disabled
                prop:checked=move || checked.get()
                on:change=move |ev| {
                    on_change.run(event_target_checked(&ev));
                }
            />
            {label.map(|lbl| {
                view! {
                    <label class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer">
                        {lbl}
                    </label>
                }
            })}
        </div>
    }
}
