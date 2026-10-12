use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[component]
pub fn Stepper(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "flex items-center w-full justify-between";
    let merged_class = tw_merge!(base, &class);

    view! {
        <ol class=merged_class>
            {children()}
        </ol>
    }
}

#[component]
pub fn Step(
    step_number: usize,
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] is_completed: bool,
    #[prop(optional)] is_current: bool,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let circle_class = if is_completed {
        "bg-primary text-primary-foreground"
    } else if is_current {
        "border-2 border-primary text-primary bg-background"
    } else {
        "border border-border text-muted-foreground bg-muted"
    };

    let base = "flex items-center gap-3";
    let merged_class = tw_merge!(base, &class);

    view! {
        <li class=merged_class>
            <span class=format!("flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-xs font-semibold {}", circle_class)>
                {if is_completed {
                    view! {
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <polyline points="20 6 9 17 4 12"></polyline>
                        </svg>
                    }.into_any()
                } else {
                    view! { <span>{step_number}</span> }.into_any()
                }}
            </span>
            <div class="flex flex-col">
                <span class="text-sm font-medium text-foreground">{title}</span>
                {description.map(|desc| {
                    view! { <span class="text-xs text-muted-foreground">{desc}</span> }
                })}
            </div>
        </li>
    }
}
