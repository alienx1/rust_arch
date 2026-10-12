use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Kbd(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100";
    let merged_class = tw_merge!(base, &class);

    view! {
        <kbd class=merged_class>
            {children()}
        </kbd>
    }
}
