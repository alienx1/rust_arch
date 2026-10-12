use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum DrawerSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl DrawerSide {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "inset-x-0 top-0 flex h-auto max-h-[70vh] flex-col overflow-y-auto rounded-b-[10px] border-b",
            Self::Bottom => {
                "inset-x-0 bottom-0 flex h-auto max-h-[70vh] flex-col overflow-y-auto rounded-t-[10px] border"
            }
            Self::Left => {
                "inset-y-0 left-0 flex h-full w-full flex-col overflow-y-auto border-r sm:max-w-md"
            }
            Self::Right => {
                "inset-y-0 right-0 flex h-full w-full flex-col overflow-y-auto border-l sm:max-w-md"
            }
        }
    }

    fn has_handle(&self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[component]
pub fn Drawer(
    open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(optional)] side: DrawerSide,
    #[prop(optional, into)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let overlay_base = "fixed inset-0 z-50 bg-black/80";
    let content_base = "fixed z-50 border-border bg-background p-4 shadow-lg";
    let merged_class = tw_merge!(content_base, side.as_str(), &class);
    let has_handle = side.has_handle();

    view! {
        <Show when=move || open.get()>
            <div class=overlay_base on:click=move |_| on_close.run(())></div>
            <div class=merged_class.clone() role="dialog" aria-modal="true">
                {if has_handle {
                    view! {
                        <div class="mx-auto mt-2 h-1.5 w-[100px] rounded-full bg-muted mb-4 cursor-pointer" on:click=move |_| on_close.run(())></div>
                    }.into_any()
                } else {
                    view! {
                        <button
                            type="button"
                            class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 cursor-pointer"
                            on:click=move |_| on_close.run(())
                        >
                            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                <line x1="6" y1="6" x2="18" y2="18"></line>
                            </svg>
                            <span class="sr-only">"Close"</span>
                        </button>
                    }.into_any()
                }}
                <div class="mx-auto flex w-full max-w-5xl flex-1 flex-col">
                    {children()}
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn DrawerHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "grid gap-1.5 p-4 text-center sm:text-left";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn DrawerTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-lg font-semibold leading-none tracking-tight";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h2 class=merged_class>
            {children()}
        </h2>
    }
}

#[component]
pub fn DrawerDescription(
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
pub fn DrawerFooter(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "mt-auto flex flex-col gap-2 p-4";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
