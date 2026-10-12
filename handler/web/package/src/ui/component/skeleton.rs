use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Skeleton(#[prop(optional, into)] class: String) -> impl IntoView {
    let base = "animate-pulse rounded-md bg-muted";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class></div>
    }
}
