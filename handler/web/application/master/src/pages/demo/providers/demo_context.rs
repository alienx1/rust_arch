//! Demo Reactive Context Provider

use leptos::prelude::*;

/// Shared reactive state context for interactive showcase components.
#[derive(Clone, Copy)]
pub struct DemoContext {
    pub active_tab: (ReadSignal<String>, WriteSignal<String>),
    pub dialog_open: (ReadSignal<bool>, WriteSignal<bool>),
    pub checkbox_val: (ReadSignal<bool>, WriteSignal<bool>),
    pub switch_val: (ReadSignal<bool>, WriteSignal<bool>),
    pub progress_val: (ReadSignal<f64>, WriteSignal<f64>),
    pub input_val: (ReadSignal<String>, WriteSignal<String>),
    pub counter: (ReadSignal<i32>, WriteSignal<i32>),
}

/// Provides DemoContext to the component tree.
pub fn provide_demo_context() -> DemoContext {
    let active_tab = signal("overview".to_string());
    let dialog_open = signal(false);
    let checkbox_val = signal(true);
    let switch_val = signal(true);
    let progress_val = signal(65.0);
    let input_val = signal("ARCH-2026".to_string());
    let counter = signal(0);

    let ctx = DemoContext {
        active_tab,
        dialog_open,
        checkbox_val,
        switch_val,
        progress_val,
        input_val,
        counter,
    };

    provide_context(ctx);
    ctx
}

/// Consume DemoContext in child components.
pub fn use_demo() -> DemoContext {
    expect_context::<DemoContext>()
}
