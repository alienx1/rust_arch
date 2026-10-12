use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Avatar(
    #[prop(optional, into)] src: Option<String>,
    #[prop(optional, into)] alt: Option<String>,
    #[prop(into)] fallback: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full border border-border bg-muted";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {match src {
                Some(url) => view! {
                    <img class="aspect-square h-full w-full object-cover" src=url alt=alt.unwrap_or_default() />
                }.into_any(),
                None => view! {
                    <span class="flex h-full w-full items-center justify-center rounded-full bg-primary/10 text-primary font-semibold text-xs uppercase">
                        {fallback}
                    </span>
                }.into_any(),
            }}
        </div>
    }
}
