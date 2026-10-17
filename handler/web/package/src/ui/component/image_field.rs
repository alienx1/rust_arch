//! Image field of a record form (logo of an organization / company, photo of an employee):
//! preview, choose a file, remove. Needs a saved record (the image belongs to its id), so a new
//! record shows a hint instead.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use super::{Button, ButtonSize, ButtonVariant, Spinner};
use crate::core::i18n::t_or;

#[component]
pub fn ImageField(
    /// Image URL of the record (`None` = no image yet)
    #[prop(into)]
    image_url: Signal<Option<String>>,
    /// The record exists (an image can only be attached to a saved record)
    #[prop(into)]
    saved: Signal<bool>,
    #[prop(into)] busy: Signal<bool>,
    #[prop(into)] can_write: Signal<bool>,
    /// First letter shown when there is no image
    #[prop(into)]
    fallback: Signal<String>,
    on_upload: Callback<web_sys::File>,
    on_remove: Callback<()>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let on_change = move |ev: leptos::ev::Event| {
        let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) else {
            return;
        };
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            on_upload.run(file);
        }
        // The same file can be chosen again after a failure
        input.set_value("");
    };

    view! {
        <div class="flex items-center gap-4">
            <div class="h-20 w-20 shrink-0 rounded-lg border border-border bg-muted/40 flex items-center justify-center overflow-hidden">
                {move || match image_url.get() {
                    Some(url) => view! { <img src=url alt="logo" class="h-full w-full object-contain" /> }.into_any(),
                    None => view! {
                        <span class="text-2xl font-bold text-muted-foreground">
                            {fallback.get().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "?".to_string())}
                        </span>
                    }.into_any(),
                }}
            </div>
            <div class="space-y-1.5 min-w-0">
                <div class="text-xs font-semibold text-muted-foreground">{t_or("common.image.label", "รูปภาพ")}</div>
                <Show
                    when=move || saved.get()
                    fallback=|| view! {
                        <div class="text-xs text-muted-foreground">
                            {t_or("common.image.save_first", "บันทึกข้อมูลก่อน แล้วจึงเพิ่มรูปภาพได้")}
                        </div>
                    }
                >
                    <div class="flex flex-wrap items-center gap-2">
                        <input node_ref=input_ref type="file" accept="image/png,image/jpeg,image/gif,image/webp" class="hidden" on:change=on_change />
                        <Show when=move || can_write.get()>
                            <Button
                                variant=ButtonVariant::Outline
                                size=ButtonSize::Sm
                                class="h-8 text-xs"
                                disabled=Signal::derive(move || busy.get())
                                on_click=Callback::new(move |_| {
                                    if let Some(input) = input_ref.get() {
                                        input.click();
                                    }
                                })
                            >
                                {move || if image_url.get().is_some() { t_or("common.image.change", "เปลี่ยนรูป") } else { t_or("common.image.choose", "เลือกรูป") }}
                            </Button>
                            <Show when=move || image_url.get().is_some()>
                                <super::confirm_button::ConfirmButton
                                    label=Signal::derive(move || t_or("common.image.remove", "ลบรูป"))
                                    confirm_text=Signal::derive(move || t_or("common.image.remove_confirm", "ลบรูปนี้ใช่หรือไม่?"))
                                    confirm_label=Signal::derive(move || t_or("common.dialog.delete_confirm", "ยืนยันลบ"))
                                    variant=ButtonVariant::Ghost
                                    class="h-8 text-xs text-destructive hover:bg-destructive/10"
                                    destructive=true
                                    busy=Signal::derive(move || busy.get())
                                    on_confirm=Callback::new(move |_| on_remove.run(()))
                                />
                            </Show>
                        </Show>
                        <Show when=move || busy.get()>
                            <Spinner class="h-4 w-4 text-primary" />
                        </Show>
                    </div>
                    <div class="text-[11px] text-muted-foreground">
                        {t_or("common.image.hint", "PNG, JPEG, GIF หรือ WebP ขนาดไม่เกิน 2 MB • รูปจะถูกบันทึกทันที")}
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// Largest image the server accepts
pub const MAX_IMAGE_BYTES: f64 = 2.0 * 1024.0 * 1024.0;

/// Reads a file chosen in the browser: its bytes and declared type, after checking what can be
/// checked before the upload (it is an image and not too big; the server judges the content)
pub async fn read_image_file(file: web_sys::File) -> Result<(Vec<u8>, String), String> {
    let content_type = file.type_();
    if !content_type.starts_with("image/") {
        return Err(t_or("common.image.error.type", "ไฟล์ต้องเป็นรูปภาพ (PNG, JPEG, GIF หรือ WebP)"));
    }
    if file.size() > MAX_IMAGE_BYTES {
        return Err(t_or("common.image.error.size", "รูปภาพต้องไม่เกิน 2 MB"));
    }
    let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| t_or("common.image.error.read", "อ่านไฟล์รูปภาพไม่สำเร็จ"))?;
    Ok((js_sys::Uint8Array::new(&buffer).to_vec(), content_type))
}
