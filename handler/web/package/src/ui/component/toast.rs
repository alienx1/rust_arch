use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub is_error: bool,
}

#[component]
pub fn Toast(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] is_error: bool,
    #[prop(optional, into)] on_dismiss: Option<Callback<()>>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let base = "pointer-events-auto relative flex w-full items-center justify-between space-x-4 overflow-hidden rounded-md border p-6 pr-8 shadow-lg transition-all";
    let variant_class = if is_error {
        "border-destructive bg-destructive text-destructive-foreground"
    } else {
        "border-border bg-background text-foreground"
    };
    let merged_class = tw_merge!(base, variant_class, &class);

    view! {
        <div class=merged_class>
            <div class="grid gap-1">
                <div class="text-sm font-semibold">{title}</div>
                {description.map(|desc| {
                    view! { <div class="text-xs opacity-90">{desc}</div> }
                })}
            </div>
            {on_dismiss.map(|cb| {
                view! {
                    <button
                        type="button"
                        class="absolute right-2 top-2 rounded-md p-1 opacity-70 transition-opacity hover:opacity-100"
                        on:click=move |_| cb.run(())
                    >
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                }
            })}
        </div>
    }
}
