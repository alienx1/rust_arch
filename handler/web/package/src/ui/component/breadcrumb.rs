use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Breadcrumb(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-wrap items-center gap-1.5 break-words text-sm text-muted-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <nav aria-label="breadcrumb">
            <ol class=merged_class>
                {children()}
            </ol>
        </nav>
    }
}

#[component]
pub fn BreadcrumbItem(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "inline-flex items-center gap-1.5";
    let merged_class = tw_merge!(base, &class);

    view! {
        <li class=merged_class>
            {children()}
        </li>
    }
}

#[component]
pub fn BreadcrumbLink(
    #[prop(into)] href: String,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "transition-colors hover:text-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <a href=href class=merged_class>
            {children()}
        </a>
    }
}

#[component]
pub fn BreadcrumbPage(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "font-normal text-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <span role="link" aria-disabled="true" aria-current="page" class=merged_class>
            {children()}
        </span>
    }
}

#[component]
pub fn BreadcrumbSeparator(#[prop(optional, into)] class: String) -> impl IntoView {
    let base = "[&>svg]:w-3.5 [&>svg]:h-3.5";
    let merged_class = tw_merge!(base, &class);

    view! {
        <li role="presentation" aria-hidden="true" class=merged_class>
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
        </li>
    }
}
