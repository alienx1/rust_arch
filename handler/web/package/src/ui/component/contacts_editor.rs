//! Editor of a record's contacts (phone, fax, e-mail, Line, tax id, ID card number): one row per
//! contact with its kind, value and "preferred" mark. The list is edited in place and the form
//! sends it with the record. A value is taken over when the field loses focus.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Button, ButtonSize, ButtonVariant};
use crate::core::i18n::t_or;

/// A contact as the API reads and writes it
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactItem {
    /// `contact_type` code: PHONE, FAX, EMAIL, LINE, TAX_ID or ID_CARD
    pub type_code: String,
    pub value: String,
    #[serde(default)]
    pub default_contact: bool,
}

const FIELD: &str = "h-9 rounded-md border border-input bg-transparent px-2 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50";

fn type_label(code: &str) -> String {
    t_or(&format!("common.contact.type.{code}"), code)
}

#[component]
pub fn ContactsEditor(
    rows: RwSignal<Vec<ContactItem>>,
    /// Kinds this record may have, in the order the picker lists them
    kinds: Vec<&'static str>,
    #[prop(into)] disabled: Signal<bool>,
) -> impl IntoView {
    let first_kind = kinds.first().copied().unwrap_or("PHONE").to_string();
    let kinds = StoredValue::new(kinds);
    let add = {
        let first_kind = first_kind.clone();
        move |_| {
            rows.update(|r| r.push(ContactItem { type_code: first_kind.clone(), ..Default::default() }));
        }
    };

    view! {
        <div class="space-y-2">
            <div class="flex items-center justify-between">
                <div>
                    <div class="text-xs font-semibold text-muted-foreground">{t_or("common.contact.title", "ช่องทางติดต่อและเลขประจำตัว")}</div>
                    <div class="text-[11px] text-muted-foreground">{t_or("common.contact.hint", "โทรศัพท์ โทรสาร อีเมล ไลน์ และเลขประจำตัว บันทึกพร้อมการกดบันทึกข้อมูล")}</div>
                </div>
                <Show when=move || !disabled.get()>
                    <Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="h-8 text-xs" on_click=Callback::new(add.clone())>
                        {t_or("common.contact.add", "+ เพิ่มช่องทางติดต่อ")}
                    </Button>
                </Show>
            </div>

            <Show
                when=move || !rows.with(|r| r.is_empty())
                fallback=|| view! { <div class="text-xs text-muted-foreground py-1">{t_or("common.contact.empty", "ยังไม่มีช่องทางติดต่อ")}</div> }
            >
                <div class="space-y-2">
                    {move || {
                        (0..rows.with(|r| r.len())).map(|i| {
                            let current = rows.with(|r| r.get(i).cloned()).unwrap_or_default();
                            let code = current.type_code.clone();
                            view! {
                                <div class="grid grid-cols-[9rem_1fr_auto_auto] items-center gap-2">
                                    <select
                                        class=FIELD
                                        disabled=move || disabled.get()
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            rows.update(|r| if let Some(x) = r.get_mut(i) { x.type_code = v });
                                        }
                                    >
                                        {kinds.get_value().into_iter().map(|k| {
                                            let selected = k == code;
                                            view! { <option value=k selected=selected>{type_label(k)}</option> }
                                        }).collect_view()}
                                    </select>
                                    <input
                                        class=format!("{FIELD} w-full")
                                        type="text"
                                        prop:value=current.value.clone()
                                        placeholder=t_or(&format!("common.contact.placeholder.{}", current.type_code), "")
                                        disabled=move || disabled.get()
                                        on:change=move |ev| {
                                            let v = event_target_value(&ev);
                                            rows.update(|r| if let Some(x) = r.get_mut(i) { x.value = v });
                                        }
                                    />
                                    <label class="flex items-center gap-1.5 text-xs text-muted-foreground whitespace-nowrap">
                                        <input
                                            type="checkbox"
                                            class="h-4 w-4"
                                            prop:checked=current.default_contact
                                            disabled=move || disabled.get()
                                            on:change=move |ev| {
                                                let on = event_target_checked(&ev);
                                                rows.update(|r| {
                                                    // One preferred contact per kind
                                                    let kind = r.get(i).map(|x| x.type_code.clone());
                                                    for (j, x) in r.iter_mut().enumerate() {
                                                        if Some(&x.type_code) == kind.as_ref() {
                                                            x.default_contact = on && j == i;
                                                        }
                                                    }
                                                });
                                            }
                                        />
                                        {t_or("common.contact.default", "ใช้เป็นหลัก")}
                                    </label>
                                    <Show when=move || !disabled.get()>
                                        <button
                                            type="button"
                                            class="h-8 px-2 text-xs text-destructive hover:bg-destructive/10 rounded-md cursor-pointer"
                                            on:click=move |_| rows.update(|r| { if i < r.len() { r.remove(i); } })
                                        >
                                            {t_or("common.contact.remove", "ลบ")}
                                        </button>
                                    </Show>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </Show>
        </div>
    }
}
