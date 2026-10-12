use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Tooltip(
    #[prop(into)] content: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let (show, set_show) = signal(false);

    let base = "relative inline-block";
    let tooltip_base = "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 z-50 overflow-hidden rounded-md bg-primary px-3 py-1.5 text-xs text-primary-foreground shadow-md animate-in fade-in-0 zoom-in-95 pointer-events-none whitespace-nowrap";
    let merged_class = tw_merge!(tooltip_base, &class);

    view! {
        <div
            class=base
            on:mouseenter=move |_| set_show.set(true)
            on:mouseleave=move |_| set_show.set(false)
        >
            {children()}
            <Show when=move || show.get()>
                <div class=merged_class.clone() role="tooltip">
                    {content.clone()}
                </div>
            </Show>
        </div>
    }
}
