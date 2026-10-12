use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn AspectRatio(
    #[prop(default = 16.0 / 9.0)] ratio: f64,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let padding_bottom = format!("{}%", (1.0 / ratio) * 100.0);
    let base = "relative w-full overflow-hidden";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class style=format!("padding-bottom: {};", padding_bottom)>
            <div class="absolute inset-0 w-full h-full">
                {children()}
            </div>
        </div>
    }
}
