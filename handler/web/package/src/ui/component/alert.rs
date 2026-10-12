use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
    Success,
    Warning,
}

impl AlertVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "bg-background text-foreground border-border",
            Self::Destructive => {
                "border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive"
            }
            Self::Success => {
                "border-emerald-500/50 text-emerald-700 dark:text-emerald-400 [&>svg]:text-emerald-600 bg-emerald-500/5"
            }
            Self::Warning => {
                "border-amber-500/50 text-amber-700 dark:text-amber-400 [&>svg]:text-amber-600 bg-amber-500/5"
            }
        }
    }
}

#[component]
pub fn Alert(
    #[prop(optional)] variant: AlertVariant,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "relative w-full rounded-lg border p-4 [&>svg~*]:pl-7 [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:text-foreground";
    let merged_class = tw_merge!(base, variant.as_str(), &class);

    view! {
        <div role="alert" class=merged_class>
            {children()}
        </div>
    }
}

#[component]
pub fn AlertTitle(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base = "mb-1 font-medium leading-none tracking-tight";
    let merged_class = tw_merge!(base, &class);

    view! {
        <h5 class=merged_class>
            {children()}
        </h5>
    }
}

#[component]
pub fn AlertDescription(
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "text-sm [&_p]:leading-relaxed";
    let merged_class = tw_merge!(base, &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
