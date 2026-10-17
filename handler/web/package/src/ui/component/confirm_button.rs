use leptos::prelude::*;

use crate::core::i18n::t_or;
use super::button::{Button, ButtonSize, ButtonVariant};

/// Two-level decision for any button that changes data: the first click asks "are you sure?" in
/// place of the button, the second one (on the confirm button) runs `on_confirm`.
///
/// `busy` disables the buttons while the request is running. `destructive` paints the question and
/// the confirm button red (deleting, suspending, forcing).
#[component]
pub fn ConfirmButton(
    /// Text of the button that opens the question
    #[prop(into)]
    label: Signal<String>,
    /// Question shown after the first click
    #[prop(optional, into)]
    confirm_text: Option<Signal<String>>,
    /// Text of the confirm button (default "ยืนยัน")
    #[prop(optional, into)]
    confirm_label: Option<Signal<String>>,
    on_confirm: Callback<()>,
    #[prop(optional, into)] busy: Signal<bool>,
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    /// Extra classes of the button that opens the question
    #[prop(optional, into)]
    class: String,
    #[prop(optional)] destructive: bool,
) -> impl IntoView {
    let (asking, set_asking) = signal(false);
    let class = StoredValue::new(class);
    let variant = variant.unwrap_or(ButtonVariant::Outline);
    let size = size.unwrap_or(ButtonSize::Sm);
    let frame = if destructive {
        "border-destructive/40 bg-destructive/5"
    } else {
        "border-primary/40 bg-primary/5"
    };
    let question_tone = if destructive { "text-destructive" } else { "text-foreground" };
    let confirm_variant = if destructive { ButtonVariant::Destructive } else { ButtonVariant::Default };

    view! {
        {move || if asking.get() {
            view! {
                <div class=format!("inline-flex flex-wrap items-center gap-2 px-3 py-1.5 rounded-md border {frame}")>
                    <span class=format!("text-xs {question_tone}")>
                        {move || match confirm_text {
                            Some(t) => t.get(),
                            None => t_or("common.dialog.confirm_question", "ยืนยันการดำเนินการนี้?"),
                        }}
                    </span>
                    <Button
                        variant=confirm_variant
                        size=ButtonSize::Sm
                        class="h-7 px-2 text-xs"
                        disabled=busy
                        on_click=Callback::new(move |_| {
                            set_asking.set(false);
                            on_confirm.run(());
                        })
                    >
                        {move || match confirm_label {
                            Some(t) => t.get(),
                            None => t_or("common.dialog.confirm", "ยืนยัน"),
                        }}
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
                    variant=variant
                    size=size
                    class=class.get_value()
                    disabled=busy
                    on_click=Callback::new(move |_| set_asking.set(true))
                >
                    {move || label.get()}
                </Button>
            }.into_any()
        }}
    }
}

/// `ConfirmButton` for saving a form: primary button, "save this data?" question
#[component]
pub fn SaveConfirmButton(
    /// Button text (usually switches to "saving..." while `busy`)
    #[prop(into)]
    label: Signal<String>,
    on_confirm: Callback<()>,
    #[prop(optional, into)] busy: Signal<bool>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let off = Signal::derive(move || busy.get() || disabled.get());
    view! {
        <ConfirmButton
            label=label
            variant=ButtonVariant::Default
            confirm_text=Signal::derive(move || t_or("common.dialog.save_question", "บันทึกข้อมูลนี้ใช่หรือไม่?"))
            confirm_label=Signal::derive(move || t_or("common.dialog.save_confirm", "ยืนยันบันทึก"))
            on_confirm=on_confirm
            busy=off
        />
    }
}
