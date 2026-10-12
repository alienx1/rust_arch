use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Switch(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    view! {
        <label class="inline-flex items-center space-x-2 cursor-pointer select-none">
            <div class="relative">
                <input
                    type="checkbox"
                    class="sr-only peer"
                    disabled=disabled
                    prop:checked=move || checked.get()
                    on:change=move |ev| {
                        on_change.run(event_target_checked(&ev));
                    }
                />
                <div class=move || {
                    let base = "w-11 h-6 bg-input peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-ring rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary";
                    tw_merge!(base, &class)
                }></div>
            </div>
            {label.map(|lbl| {
                view! {
                    <span class="text-sm font-medium text-foreground">
                        {lbl}
                    </span>
                }
            })}
        </label>
    }
}
