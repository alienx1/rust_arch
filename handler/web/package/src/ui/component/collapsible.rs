use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Collapsible(
    #[prop(into)] open: Signal<bool>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context(open);

    let base = "w-full";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn CollapsibleTrigger(
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
pub fn CollapsibleContent(
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let open =
        use_context::<Signal<bool>>().expect("CollapsibleContent must be inside Collapsible");

    let base = "overflow-hidden transition-all";
    let merged_class = tw_merge!(base, &class);

    view! {
        <Show when=move || open.get()>
            <div class=merged_class.clone()>
                {children()}
            </div>
        </Show>
    }
}
