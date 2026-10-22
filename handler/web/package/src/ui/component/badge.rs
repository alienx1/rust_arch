use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
}

impl BadgeVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => {
                "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
            }
            Self::Secondary => {
                "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
            }
            Self::Destructive => {
                "border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80"
            }
            Self::Outline => "text-foreground border border-border",
            Self::Success => {
                "border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 font-medium"
            }
            Self::Warning => {
                "border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-400 font-medium"
            }
        }
    }
}

#[component]
pub fn Badge(
    #[prop(optional)] variant: BadgeVariant,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    let base = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";
    let merged_class = tw_merge!(base, variant.as_str(), &class);

    view! {
        <div class=merged_class>
            {children()}
        </div>
    }
}
