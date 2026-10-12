use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Pagination(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "mx-auto flex w-full justify-center";
    let merged_class = tw_merge!(base, &class);

    view! {
        <nav role="navigation" aria-label="pagination" class=merged_class>
            {children()}
        </nav>
    }
}

#[component]
pub fn PaginationContent(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "flex flex-row items-center gap-1";
    let merged_class = tw_merge!(base, &class);

    view! {
        <ul class=merged_class>
            {children()}
        </ul>
    }
}

#[component]
pub fn PaginationItem(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "";
    let merged_class = tw_merge!(base, &class);

    view! {
        <li class=merged_class>
            {children()}
        </li>
    }
}

#[component]
pub fn PaginationLink(
    #[prop(into)] href: String,
    #[prop(optional)] is_active: bool,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 h-9 w-9";
    let active_class = if is_active {
        "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground"
    } else {
        "hover:bg-accent hover:text-accent-foreground"
    };
    let merged_class = tw_merge!(base, active_class, &class);

    view! {
        <a href=href class=merged_class aria-current=if is_active { "page" } else { "false" }>
            {children()}
        </a>
    }
}

#[component]
pub fn PaginationPrevious(
    #[prop(into)] href: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 gap-1 pl-2.5";
    let merged_class = tw_merge!(base, &class);

    view! {
        <a href=href class=merged_class aria-label="Go to previous page">
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
            <span>"Previous"</span>
        </a>
    }
}

#[component]
pub fn PaginationNext(
    #[prop(into)] href: String,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 gap-1 pr-2.5";
    let merged_class = tw_merge!(base, &class);

    view! {
        <a href=href class=merged_class aria-label="Go to next page">
            <span>"Next"</span>
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
        </a>
    }
}

#[component]
pub fn PaginationEllipsis(#[prop(optional, into)] class: String) -> impl IntoView {
    let base = "flex h-9 w-9 items-center justify-center";
    let merged_class = tw_merge!(base, &class);

    view! {
        <span aria-hidden="true" class=merged_class>
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="1"></circle>
                <circle cx="19" cy="12" r="1"></circle>
                <circle cx="5" cy="12" r="1"></circle>
            </svg>
            <span class="sr-only">"More pages"</span>
        </span>
    }
}
