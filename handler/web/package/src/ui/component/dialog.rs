use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Dialog(
    open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let overlay_base = "fixed inset-0 z-50 bg-black/80 backdrop-blur-sm transition-all duration-100 data-[state=closed]:animate-out data-[state=closed]:fade-out data-[state=open]:fade-in";
    let content_base = "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border border-border bg-background p-6 shadow-lg duration-200 sm:rounded-lg";
    let merged_content = tw_merge!(content_base, &class);

    view! {
        <Show when=move || open.get()>
            <div class=overlay_base on:click=move |_| on_close.run(())></div>
            <div class=merged_content.clone() role="dialog" aria-modal="true">
                <button
                    type="button"
                    class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none cursor-pointer"
                    on:click=move |_| on_close.run(())
                >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                    <span class="sr-only">"Close"</span>
                </button>
                {children()}
            </div>
        </Show>
    }
}

#[component]
pub fn DialogHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-col space-y-1.5 text-center sm:text-left";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn DialogTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-lg font-semibold leading-none tracking-tight";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h2 class=merged_class>
            {children()}
        </h2>
    }
}

#[component]
pub fn DialogDescription(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "text-sm text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <p class=merged_class>
            {children()}
        </p>
    }
}

#[component]
pub fn DialogFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
