use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Empty(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex min-h-[400px] flex-col items-center justify-center rounded-md border border-dashed border-border p-8 text-center animate-in fade-in-50";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            <div class="mx-auto flex max-w-[420px] flex-col items-center justify-center text-center">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn EmptyIcon(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex h-20 w-20 items-center justify-center rounded-full bg-muted text-muted-foreground mb-4";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn EmptyTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-xl font-semibold text-foreground mb-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h3 class=merged_class>
            {children()}
        </h3>
    }
}

#[component]
pub fn EmptyDescription(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "text-sm text-muted-foreground mb-6 leading-relaxed";
    let merged_class = tw_merge!(base, &class);

    view! {
        <p class=merged_class>
            {children()}
        </p>
    }
}

#[component]
pub fn EmptyActions(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex items-center gap-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
