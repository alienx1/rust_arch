use leptos::prelude::*;

use crate::core::i18n::t_or;
use super::button::{Button, ButtonSize, ButtonVariant};

/// Two-step delete: the first click asks "are you sure?" in place, the second one confirms.
///
/// `busy` disables both buttons while the request is running.
#[component]
pub fn DeleteConfirmButton(
    /// Button text, e.g. "ลบข้อมูล"
    #[prop(into)]
    label: String,
    /// Question shown after the first click
    #[prop(into)]
    confirm_text: String,
    on_confirm: Callback<()>,
    #[prop(optional, into)] busy: Option<Signal<bool>>,
) -> impl IntoView {
    let (asking, set_asking) = signal(false);
    let busy = busy.unwrap_or_else(|| Signal::stored(false));
    let (label, confirm_text) = (StoredValue::new(label), StoredValue::new(confirm_text));

    view! {
        {move || if asking.get() {
            view! {
                <div class="flex items-center gap-2 px-3 py-1.5 rounded-md border border-destructive/40 bg-destructive/5">
                    <span class="text-xs text-destructive">{confirm_text.get_value()}</span>
                    <Button
                        variant=ButtonVariant::Destructive
                        size=ButtonSize::Sm
                        class="h-7 px-2 text-xs"
                        disabled=busy
                        on_click=Callback::new(move |_| {
                            set_asking.set(false);
                            on_confirm.run(());
                        })
                    >
                        {move || t_or("common.dialog.delete_confirm", "ยืนยันลบ")}
                    </Button>
                    <Button
                        variant=ButtonVariant::Outline
                        size=ButtonSize::Sm
                        class="h-7 px-2 text-xs"
                        on_click=Callback::new(move |_| set_asking.set(false))
                    >
                        {move || t_or("common.action.cancel", "ยกเลิก")}
                    </Button>
                </div>
            }.into_any()
        } else {
            view! {
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Sm
                    class="text-destructive border-destructive/40 hover:bg-destructive/10"
                    disabled=busy
                    on_click=Callback::new(move |_| set_asking.set(true))
                >
                    {label.get_value()}
                </Button>
            }.into_any()
        }}
    }
}
