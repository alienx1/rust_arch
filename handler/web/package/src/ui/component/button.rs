use leptos::prelude::*;
use tailwind_fuse::tw_merge;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground shadow hover:bg-primary/90",
            Self::Destructive => {
                "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90"
            }
            Self::Outline => {
                "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground"
            }
            Self::Secondary => {
                "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80"
            }
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground",
            Self::Link => "text-primary underline-offset-4 hover:underline",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

impl ButtonSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Sm => "h-8 rounded-md px-3 text-xs",
            Self::Lg => "h-10 rounded-md px-8",
            Self::Icon => "h-9 w-9 p-0",
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional, into)] class: String,
    #[prop(optional, into)] r#type: Option<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    let button_type = r#type.unwrap_or_else(|| "button".to_string());
    let base = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 cursor-pointer";
    let merged_class = tw_merge!(base, variant.as_str(), size.as_str(), &class);

    view! {
        <button
            type=button_type
            class=merged_class
            prop:disabled=move || disabled.get()
            on:click=move |ev| {
                if let Some(cb) = &on_click {
                    cb.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
