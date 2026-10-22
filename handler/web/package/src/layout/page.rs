use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn PageContainer(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex-1 overflow-y-auto p-4 sm:p-6 lg:p-8 space-y-6 w-full";
    let merged_class = tw_merge!(base, &class);

    view! {
        <main class=merged_class>
            {children()}
        </main>
    }
}

#[component]
pub fn PageHeader(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-col gap-4 md:flex-row md:items-center md:justify-between pb-6 border-b border-border";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn PageTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-xl sm:text-2xl font-bold tracking-tight text-foreground";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h1 class=merged_class>
            {children()}
        </h1>
    }
}

#[component]
pub fn PageDescription(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "text-sm text-muted-foreground mt-1";
    let merged_class = tw_merge!(base, &class);

    view! {
        <p class=merged_class>
            {children()}
        </p>
    }
}

#[component]
pub fn PageActions(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex flex-wrap items-center gap-3";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn PageContent(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "space-y-6";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
