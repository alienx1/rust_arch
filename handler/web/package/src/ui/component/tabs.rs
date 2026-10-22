use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Tabs(
    #[prop(into)] active_tab: Signal<String>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "flex flex-col gap-2";
    let merged_class = tw_merge!(base, &class);

    // Provide the active tab context
    provide_context(active_tab);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn TabsList(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base =
        "inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="tablist">
            {children()}
        </div>
    }
}

#[component]
pub fn TabsTrigger(
    #[prop(into)] value: String,
    #[prop(into)] on_select: Callback<String>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let active_tab = use_context::<Signal<String>>().expect("TabsTrigger must be inside Tabs");
    let val_for_class = value.clone();
    let val_for_aria = value.clone();
    let val_for_click = value;

    let base = "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 cursor-pointer";

    let current_class = move || {
        if active_tab.get() == val_for_class {
            tw_merge!(base, "bg-card text-foreground shadow-sm", &class)
        } else {
            tw_merge!(base, "hover:text-foreground", &class)
        }
    };

    view! {
        <button
            type="button"
            role="tab"
            class=current_class
            aria-selected=move || (active_tab.get() == val_for_aria).to_string()
            on:click=move |_| on_select.run(val_for_click.clone())
        >
            {children()}
        </button>
    }
}

#[component]
pub fn TabsContent(
    #[prop(into)] value: String,
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let active_tab = use_context::<Signal<String>>().expect("TabsContent must be inside Tabs");
    let is_active = move || active_tab.get() == value;

    let base = "mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <Show when=is_active>
            <div class=merged_class.clone() role="tabpanel">
                {children()}
            </div>
        </Show>
    }
}
