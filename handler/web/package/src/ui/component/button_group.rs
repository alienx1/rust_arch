use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn ButtonGroup(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "inline-flex rounded-md shadow-sm -space-x-px [&>button]:rounded-none [&>button:first-child]:rounded-l-md [&>button:last-child]:rounded-r-md [&>button:focus]:z-10";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="group">
            {children()}
        </div>
    }
}
