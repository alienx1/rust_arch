use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn AppShell(
    #[prop(optional, into)] class: String,
    #[prop(into)] sidebar: ViewFn,
    #[prop(into)] header: ViewFn,
    children: Children,
) -> impl IntoView {
    let base = "flex h-dvh w-full overflow-hidden bg-background";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            // Sidebar area
            {sidebar.run()}

            // Main content area with header
            <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
                {header.run()}
                <div class="flex-1 overflow-y-auto">
                    {children()}
                </div>
            </div>
        </div>
    }
}
