use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Accordion(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "divide-y divide-border rounded-md border border-border";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn AccordionItem(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let is_open = signal(false);
    provide_context(is_open);

    let base = "border-b border-border last:border-b-0";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn AccordionTrigger(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let (is_open, set_is_open) = use_context::<(ReadSignal<bool>, WriteSignal<bool>)>()
        .expect("AccordionTrigger must be inside AccordionItem");

    let base = "flex flex-1 items-center justify-between py-4 px-4 font-medium transition-all hover:underline cursor-pointer select-none text-sm";
    let merged_class = tw_merge!(base, &class);

    view! {
        <button
            type="button"
            class=merged_class
            on:click=move |_| set_is_open.update(|open| *open = !*open)
        >
            {children()}
            <svg
                class=move || {
                    let rot = if is_open.get() { "rotate-180" } else { "rotate-0" };
                    tw_merge!("h-4 w-4 shrink-0 transition-transform duration-200 text-muted-foreground", rot)
                }
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
            </svg>
        </button>
    }
}

#[component]
pub fn AccordionContent(
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let (is_open, _) = use_context::<(ReadSignal<bool>, WriteSignal<bool>)>()
        .expect("AccordionContent must be inside AccordionItem");

    let base = "pb-4 pt-0 px-4 text-sm text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <Show when=move || is_open.get()>
            <div class=merged_class.clone()>
                {children()}
            </div>
        </Show>
    }
}
