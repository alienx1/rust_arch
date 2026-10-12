use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SheetSide {
    Top,
    Bottom,
    Left,
    #[default]
    Right,
}

impl SheetSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "inset-x-0 top-0 border-b",
            Self::Bottom => "inset-x-0 bottom-0 border-t",
            Self::Left => "inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm",
            Self::Right => "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm",
        }
    }
}

#[component]
pub fn Sheet(
    open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional)] side: SheetSide,
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let overlay_base =
        "fixed inset-0 z-50 bg-black/80 backdrop-blur-sm transition-all duration-100";
    let sheet_base =
        "fixed z-50 gap-4 bg-background p-6 shadow-lg transition ease-in-out border-border";
    let merged_class = tw_merge!(sheet_base, side.as_str(), &class);

    view! {
        <Show when=move || open.get()>
            <div class=overlay_base on:click=move |_| on_close.run(())></div>
            <div class=merged_class.clone() role="dialog" aria-modal="true">
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
pub fn SheetHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-col space-y-2 text-center sm:text-left";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn SheetTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-lg font-semibold text-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h2 class=merged_class>
            {children()}
        </h2>
    }
}

#[component]
pub fn SheetDescription(
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
pub fn SheetFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
