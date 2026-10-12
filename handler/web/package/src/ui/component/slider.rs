use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Slider(
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] on_change: Callback<f64>,
    #[prop(default = 0.0)] min: f64,
    #[prop(default = 100.0)] max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let base = "relative flex w-full touch-none select-none items-center cursor-pointer";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            <input
                type="range"
                min=min
                max=max
                step=step
                disabled=disabled
                class="w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary"
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                        on_change.run(val);
                    }
                }
            />
        </div>
    }
}
