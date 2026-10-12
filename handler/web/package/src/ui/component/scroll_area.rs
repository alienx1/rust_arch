use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn ScrollArea(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "relative overflow-auto scrollbar-thin scrollbar-thumb-muted-foreground/20 scrollbar-track-transparent";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
