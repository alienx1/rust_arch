use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn AlertDialog(open: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm transition-all duration-100 data-[state=closed]:animate-out data-[state=closed]:fade-out data-[state=open]:fade-in"></div>
            {children()}
        </Show>
    }
}

#[component]
pub fn AlertDialogContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border border-border bg-background p-6 shadow-lg duration-200 sm:rounded-lg";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class role="alertdialog" aria-modal="true">
            {children()}
        </div>
    }
}

#[component]
pub fn AlertDialogHeader(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "flex flex-col space-y-2 text-center sm:text-left";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn AlertDialogTitle(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "text-lg font-semibold";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h2 class=merged_class>
            {children()}
        </h2>
    }
}

#[component]
pub fn AlertDialogDescription(
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
pub fn AlertDialogFooter(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
