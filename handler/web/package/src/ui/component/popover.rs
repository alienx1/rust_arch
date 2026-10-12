use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Popover(
    #[prop(into)] open: Signal<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context(open);

    let base = "relative inline-block text-left";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn PopoverTrigger(
    #[prop(into)] on_toggle: Callback<()>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "cursor-pointer";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class on:click=move |_| on_toggle.run(())>
            {children()}
        </div>
    }
}

#[component]
pub fn PopoverContent(
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let open = use_context::<Signal<bool>>().expect("PopoverContent must be inside Popover");

    let base = "absolute z-50 mt-2 w-72 rounded-md border border-border bg-popover p-4 text-popover-foreground shadow-md outline-none animate-in fade-in-0 zoom-in-95";
    let merged_class = tw_merge!(base, &class);

    view! {
        <Show when=move || open.get()>
            <div class=merged_class.clone() tabindex="-1">
                {children()}
            </div>
        </Show>
    }
}
