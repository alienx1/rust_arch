use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Progress(
    #[prop(into)] value: Signal<f64>,
    #[prop(default = 100.0)] max: f64,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "relative h-2 w-full overflow-hidden rounded-full bg-primary/20";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="progressbar" aria-valuemin="0" aria-valuemax=max aria-valuenow=move || value.get()>
            <div
                class="h-full w-full flex-1 bg-primary transition-all duration-300"
                style=move || {
                    let percent = (value.get() / max).clamp(0.0, 1.0) * 100.0;
                    format!("transform: translateX(-{}%);", 100.0 - percent)
                }
            ></div>
        </div>
    }
}
