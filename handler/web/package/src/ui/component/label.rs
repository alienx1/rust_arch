use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Label(
    #[prop(optional, into)] r#for: Option<String>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <label for=r#for class=merged_class>
            {children()}
        </label>
    }
}
