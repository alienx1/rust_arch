use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Separator(
    #[prop(optional)] orientation: Option<&'static str>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let is_vertical = orientation == Some("vertical");
    let base = if is_vertical {
        "shrink-0 bg-border h-full w-[1px]"
    } else {
        "shrink-0 bg-border h-[1px] w-full"
    };
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="separator"></div>
    }
}
