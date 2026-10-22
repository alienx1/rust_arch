use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Header(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "sticky top-0 z-20 flex h-14 w-full items-center justify-between border-b border-border bg-background/80 px-4 backdrop-blur-md";
    let merged_class = tw_merge!(base, &class);

    view! {
        <header class=merged_class>
            {children()}
        </header>
    }
}

#[component]
pub fn HeaderLeft(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex min-w-0 items-center gap-2 sm:gap-4";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn HeaderRight(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex shrink-0 items-center gap-3";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
