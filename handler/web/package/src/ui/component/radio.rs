use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn RadioGroup(
    #[prop(into)] value: Signal<String>,
    #[prop(into)] on_change: Callback<String>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context((value, on_change));

    let base = "grid gap-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="radiogroup">
            {children()}
        </div>
    }
}

#[component]
pub fn RadioGroupItem(
    #[prop(into)] value: String,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let (current_val, on_change) = use_context::<(Signal<String>, Callback<String>)>()
        .expect("RadioGroupItem must be inside RadioGroup");

    let item_val = value.clone();
    let is_checked = move || current_val.get() == item_val;

    let base = "aspect-square h-4 w-4 rounded-full border border-primary text-primary shadow focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 accent-primary cursor-pointer";
    let merged_class = tw_merge!(base, &class);

    let click_val = value;
    view! {
        <div class="flex items-center space-x-2">
            <input
                type="radio"
                class=merged_class
                disabled=disabled
                prop:checked=is_checked
                on:change=move |_| on_change.run(click_val.clone())
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
